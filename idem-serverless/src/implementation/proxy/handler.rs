use crate::entry::LambdaExchange;
use crate::implementation::proxy::config::LambdaProxyHandlerConfig;
use crate::implementation::Handler;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use idem_config::config::Config;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::future::Future;
use std::ops::Add;
use std::pin::Pin;

pub const FUNCTION_NAME_SEPARATOR: &str = "@";

pub struct LambdaProxyHandler {
    config: Config<LambdaProxyHandlerConfig>,
}

impl LambdaProxyHandler {
    pub fn new(config: Config<LambdaProxyHandlerConfig>) -> Self {
        Self { config }
    }
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for LambdaProxyHandler {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut LambdaExchange,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        Box::pin(async move {
            let client =
                LambdaClient::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);
            if !self.config.get().enabled {
                return Ok(HandlerStatus::new(Code::DISABLED));
            }

            match exchange.consume_request() {
                Ok(request) => {
                    let payload = serde_json::to_string(&request).unwrap();
                    let path = match request.path {
                        Some(path) => path,
                        None => todo!("Handle path not found in request"),
                    };
                    let method = request.http_method;
                    let function_key = path.add(FUNCTION_NAME_SEPARATOR).add(method.as_str());
                    let function_name = match self.config.get().functions.get(&function_key) {
                        None => todo!("Handle no function found matching in configuration"),
                        Some(function) => function.clone(),
                    };
                    let proxy_blob = Blob::new(payload);
                    match client
                        .invoke()
                        .function_name(&function_name)
                        .payload(proxy_blob)
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.function_error().is_some() {
                                todo!("Handle function failure")
                            }

                            let response_payload_bytes = response.payload.unwrap().into_inner();
                            let lambda_response: ApiGatewayProxyResponse =
                                serde_json::from_slice(&response_payload_bytes).unwrap_or_else(
                                    |_| todo!("failed to get response from lambda function call."),
                                );
                            exchange.save_output(lambda_response);
                            println!("LambdaProxyHandler successfully finished!");
                            Ok(HandlerStatus::new(Code::REQUEST_COMPLETED))
                        }
                        Err(_) => Ok(HandlerStatus::new(Code::SERVER_ERROR)),
                    }
                }
                Err(_) => Ok(HandlerStatus::new(Code::SERVER_ERROR)),
            }
        })
    }
}
