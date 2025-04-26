use crate::implementation::proxy::config::LambdaProxyHandlerConfig;
use crate::implementation::proxy::handler::LambdaProxyHandler;
use crate::implementation::traceability::config::TraceabilityHandlerConfig;
use crate::implementation::traceability::handler::TraceabilityHandler;
use crate::implementation::{LambdaHandlerExecutor, LambdaHandlers};
use idem_config::config::{Config, DefaultConfigProvider};
use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::Code;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use std::fmt::Display;

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;

pub(crate) async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut handlers: Vec<LambdaHandlers> = vec![];

    let traceability_config: Config<TraceabilityHandlerConfig> =
        Config::new(DefaultConfigProvider).unwrap();
    handlers.push(LambdaHandlers::TraceabilityHandler(
        TraceabilityHandler::new(traceability_config),
    ));

    let proxy_config: Config<LambdaProxyHandlerConfig> =
        Config::new(DefaultConfigProvider).unwrap();
    handlers.push(LambdaHandlers::ProxyHandler(LambdaProxyHandler::new(
        proxy_config,
    )));

    let executor = LambdaHandlerExecutor::new(handlers);
    let (payload, context) = event.into_parts();

    let mut exchange: LambdaExchange = Exchange::new();
    exchange.save_input(payload);
    exchange.add_metadata(context);

    'handler_exec: for handler in &executor.handlers {
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
                todo!("Return with exception handler")
            }
        }
    }

    Ok(exchange.consume_output().unwrap())
}
