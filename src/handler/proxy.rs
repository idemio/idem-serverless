use std::collections::HashMap;
use serde::{Deserialize};
use std::ops::Add;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use crate::handler::LambdaExchange;

#[derive(Deserialize, Default)]
pub(crate) struct LambdaProxyHandlerConfig {
    pub enabled: bool,
    pub functions: HashMap<String, String>
}



const FUNCTION_NAME_SEPARATOR: &str = "@";

#[derive(ConfigurableHandler)]
pub struct LambdaProxyHandler {
    config: Config<LambdaProxyHandlerConfig>,
}

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for LambdaProxyHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError>

    {
        let client =
            LambdaClient::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        match exchange.take_request() {
            Ok(request) => {
                let payload = serde_json::to_string(&request).unwrap();
                let path = match request.path {
                    Some(path) => path,
                    _ => {
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
    }
}
