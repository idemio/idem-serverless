use crate::entry::LambdaExchange;
use crate::implementation::proxy::config::LambdaProxyHandlerConfig;
use crate::implementation::{Handler, HandlerOutput};
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use idem_config::config::Config;
use idem_handler::status::{Code, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::ops::Add;

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
    fn exec<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut LambdaExchange,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result,
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
                        None => {
                            return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                                .set_message("Missing path in request."))
                        }
                    };
                    let method = request.http_method;
                    let function_key = path.add(FUNCTION_NAME_SEPARATOR).add(method.as_str());
                    let function_name = match self.config.get().functions.get(&function_key) {
                        None => {
                            return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                                .set_message("No function found for path and method combination."))
                        }
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

                                return Ok(HandlerStatus::new(Code::SERVER_ERROR)
                                    .set_message("Lambda function returned an error."));
                            }

                            let response_payload_bytes = response.payload.unwrap().into_inner();
                            let lambda_response: ApiGatewayProxyResponse =
                                match serde_json::from_slice(&response_payload_bytes) {
                                    Ok(response) => response,
                                    Err(_) => {
                                        return Ok(HandlerStatus::new(Code::SERVER_ERROR)
                                            .set_message(
                                                "Failed to parse response from Lambda function.",
                                            ));
                                    }
                                };
                            exchange.save_output(lambda_response);
                            Ok(HandlerStatus::new(Code::REQUEST_COMPLETED))
                        }
                        Err(_) => Ok(HandlerStatus::new(Code::SERVER_ERROR)
                            .set_message("Failed to invoke Lambda function.")),
                    }
                }
                Err(_) => Ok(HandlerStatus::new(Code::SERVER_ERROR)
                    .set_message("Failed to consume request.")),
            }
        })
    }
}
