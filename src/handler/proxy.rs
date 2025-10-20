use std::collections::HashMap;
use std::convert::Infallible;
use serde::{Deserialize};
use std::ops::Add;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use idemio::config::Config;
use idemio::exchange::Exchange;
use idemio::handler::Handler;
use idemio::status::{ExchangeState, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use crate::handler::LambdaExchange;

#[derive(Deserialize, Default)]
pub(crate) struct LambdaProxyHandlerConfig {
    pub enabled: bool,
    pub functions: HashMap<String, String>
}



const FUNCTION_NAME_SEPARATOR: &str = "@";

//#[derive(ConfigurableHandler)]
pub struct LambdaProxyHandler {
    config: Config<LambdaProxyHandlerConfig>,
}

#[async_trait]
impl Handler<Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>> for LambdaProxyHandler {

    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, Infallible>

    {
        let client =
            LambdaClient::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(ExchangeState::DISABLED));
        }

        match exchange.take_input().await {
            Ok(request) => {
                let payload = serde_json::to_string(&request).unwrap();
                let path = match request.path {
                    Some(path) => path,
                    _ => {
                        return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                            .message("Missing path in request."))
                    }
                };
                let method = request.http_method;
                let function_key = path.add(FUNCTION_NAME_SEPARATOR).add(method.as_str());
                let function_name = match self.config.get().functions.get(&function_key) {
                    None => {
                        return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                            .message("No function found for path and method combination."))
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

                            return Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR)
                                .message("Lambda function returned an error."));
                        }

                        let response_payload_bytes = response.payload.unwrap().into_inner();
                        let lambda_response: ApiGatewayProxyResponse =
                            match serde_json::from_slice(&response_payload_bytes) {
                                Ok(response) => response,
                                Err(_) => {
                                    return Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR)
                                        .message(
                                            "Failed to parse response from Lambda function.",
                                        ));
                                }
                            };
                        exchange.set_output(lambda_response);
                        Ok(HandlerStatus::new(ExchangeState::EXCHANGE_COMPLETED))
                    }
                    Err(_) => Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR)
                        .message("Failed to invoke Lambda function.")),
                }
            }
            Err(_) => Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR)
                .message("Failed to consume request.")),
        }
    }

    fn name(&self) -> &str {
        "LambdaProxyHandler"
    }
}
