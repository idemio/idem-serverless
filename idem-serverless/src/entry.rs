use std::fs::File;
use crate::implementation::{LambdaHandlerExecutor, LambdaHandlerFactory};
use idem_config::config::{ ProviderType};
use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::Code;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use idem_config::execution_flow_config::ExecutionFlowConfig;

pub async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Load the execution flow configuration
    let config_file = File::open("/opt/config/handlers.json").unwrap();
    let execution_flow_config: ExecutionFlowConfig = serde_json::from_reader(config_file).unwrap();
    let (payload, context) = event.into_parts();

    let path = match &payload.path {
        None => todo!(),
        Some(path) => path.clone()
    };
    let method = &payload.http_method.clone();

    // Find the matching path configuration
    if let Some(path_config) = execution_flow_config.paths.get(&path) {
        if path_config.method.eq_ignore_ascii_case(method.as_str()) {
            let mut handlers_to_execute = vec![];

            // Resolve handlers and chains
            for exec in &path_config.exec {
                if let Some(chain) = execution_flow_config.chains.get(exec) {
                    // Add all handlers from the chain
                    for handler_name in chain {
                        if let Ok(handler) = LambdaHandlerFactory::create_handler(handler_name, ProviderType::File) {
                            handlers_to_execute.push(handler);
                        }
                    }
                } else if let Ok(handler) = LambdaHandlerFactory::create_handler(exec, ProviderType::File) {
                    // Add individual handler
                    handlers_to_execute.push(handler);
                }
            }

            // Build the executor with dynamically loaded handlers
            let executor = LambdaHandlerExecutor::new(handlers_to_execute);

            let mut exchange: LambdaExchange = Exchange::new();
            exchange.save_input(payload);
            exchange.add_metadata(context);

            // Execute handlers
            'handler_exec: for handler in executor.handlers {
                match handler.process(&mut exchange).await {
                    Ok(status) => {
                        if status
                            .code()
                            .any_flags(Code::TIMEOUT | Code::SERVER_ERROR | Code::CLIENT_ERROR)
                        {
                            todo!("Handle exception here")
                        } else if status.code().any_flags(Code::CONTINUE) {
                            todo!("Handle continue flow here")
                        } else if status.code().any_flags(Code::OK | Code::DISABLED) {
                            continue;
                        } else if status.code().all_flags(Code::REQUEST_COMPLETED) {
                            break 'handler_exec;
                        }
                    }
                    Err(_err) => {
                        return Err(Error::from("Handler execution failed"));
                    }
                }
            }

            return Ok(exchange.consume_output().unwrap());
        }
    }

    // If no matching path or method found
    Err(Error::from(format!(
        "No configuration found for path: {} and method: {}",
        path, method
    )))
}


pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;

//pub async fn entry(
//    event: LambdaEvent<ApiGatewayProxyRequest>,
//) -> Result<ApiGatewayProxyResponse, Error> {
//
//
//    let config_file = File::open("./config/handlers.json").unwrap();
//    let execution_flow_config: ExecutionFlowConfig = serde_json::from_reader(config_file).unwrap();
//
//
//    let mut handlers: Vec<LambdaHandler> = vec![];
//
//    let traceability_config: Config<TraceabilityHandlerConfig> =
//        Config::new(DefaultConfigProvider).unwrap();
//    handlers.push(LambdaHandler::TraceabilityHandler(
//        TraceabilityHandler::new(traceability_config),
//    ));
//
//    let proxy_config: Config<LambdaProxyHandlerConfig> =
//        Config::new(DefaultConfigProvider).unwrap();
//    handlers.push(LambdaHandler::ProxyHandler(LambdaProxyHandler::new(
//        proxy_config,
//    )));
//
//    let executor = LambdaHandlerExecutor::new(handlers);
//    let (payload, context) = event.into_parts();
//
//    let mut exchange: LambdaExchange = Exchange::new();
//    exchange.save_input(payload);
//    exchange.add_metadata(context);
//
//    'handler_exec: for handler in &executor.handlers {
//        match handler.process(&mut exchange).await {
//            Ok(status) => {
//                if status
//                    .code()
//                    .any_flags(Code::TIMEOUT | Code::SERVER_ERROR | Code::CLIENT_ERROR)
//                {
//                    todo!("Handle exception here")
//                } else if status.code().any_flags(Code::CONTINUE) {
//                    todo!("Handle continue flow here")
//                } else if status.code().any_flags(Code::OK | Code::DISABLED) {
//                    continue;
//                } else if status.code().all_flags(Code::REQUEST_COMPLETED) {
//                    break 'handler_exec;
//                }
//            }
//            Err(_err) => {
//                todo!("Return with exception handler")
//            }
//        }
//    }
//
//    Ok(exchange.consume_output().unwrap())
//}
