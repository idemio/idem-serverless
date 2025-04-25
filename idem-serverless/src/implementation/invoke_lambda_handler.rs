use idem_config::config::{Config, ConfigResult};
use crate::implementation::Handler;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Add;
use std::pin::Pin;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use crate::entry::LambdaExchange;

pub const FUNCTION_NAME_SEPARATOR: &str = "@";

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct LambdaProxyHandlerConfig {
    pub enabled: bool,
    pub functions: HashMap<String, String>,
    pub region: String,
    pub endpoint_override: String,
    pub api_call_timeout: u32,
    pub log_type: String,
    pub metrics_injection: bool,
    pub metrics_name: String,
}

impl Config for LambdaProxyHandlerConfig {
    fn load_local_file() -> ConfigResult<Self> {
        todo!("impl load local")
    }

    fn load_programmatically() -> ConfigResult<Self> {
        todo!("impl load programmatically")
    }

    fn load_remote() -> ConfigResult<Self> {
        todo!("impl load remote")
    }
}

#[derive(Clone, Default)]
pub(crate) struct LambdaProxyHandler {
    lambda_client: Option<LambdaClient>,
    config: LambdaProxyHandlerConfig,
}

impl LambdaProxyHandler {
    pub async fn new(config: LambdaProxyHandlerConfig) -> Self {
        Self {
            lambda_client: Some(LambdaClient::new(
                &aws_config::load_defaults(BehaviorVersion::latest()).await,
            )),
            config,
        }
    }
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>
    for LambdaProxyHandler
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut LambdaExchange,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        println!("Proxy handler starts!");
        let client = self.lambda_client.clone();
        Box::pin(async move {

            if !self.config.enabled {
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
                    let function_name = match self.config.functions.get(&function_key) {
                        None => todo!("Handle no function found matching in configuration"),
                        Some(function) => function.clone(),
                    };
                    let proxy_blob = Blob::new(payload);
                    match client
                        .unwrap()
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
