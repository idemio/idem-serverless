use crate::config::config::{Config, ConfigResult};
use crate::exchange::Exchange;
use crate::handlers::Handler;
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

pub const FUNCTION_NAME_SEPARATOR: &str = "@";

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct LambdaProxyHandlerConfig {
    enabled: bool,
    functions: HashMap<String, String>,
    region: String,
    endpoint_override: String,
    api_call_timeout: u32,
    log_type: String,
    metrics_injection: bool,
    metrics_name: String,
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
    pub(crate) async fn new(config: LambdaProxyHandlerConfig) -> Self {
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
        exchange: &'i2 mut Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        let client = self.lambda_client.clone();
        Box::pin(async move {
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
                            Ok(())
                        }
                        Err(_) => todo!("Handle SDK error"),
                    }
                }
                Err(_) => todo!("Handle no request failure"),
            }
        })
    }
}
