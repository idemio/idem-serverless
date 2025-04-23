use crate::exchange::Exchange;
use crate::handlers::Handler;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use aws_sdk_lambda::primitives::Blob;
use lambda_http::http::header::CONTENT_TYPE;

const HEALTH_STATUS: u32 = 200u32;
const HEALTH_BODY: &str = "OK";
const HEALTH_ERROR: &str = "ERROR";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AWSHealCheckHandlerConfig {
    enabled: bool,
    use_json: bool,
    timeout: u32,
    downstream_enabled: bool,
    downstream_function: String,
    downstream_function_health_payload: String,
}

#[derive(Clone, Default)]
pub struct AWSHealthCheckHandler {
    lambda_client: Option<LambdaClient>,
    config: AWSHealCheckHandlerConfig,
}

impl AWSHealthCheckHandler {
    pub(crate) async fn new(config: AWSHealCheckHandlerConfig) -> Self {
        Self {
            lambda_client: Some(LambdaClient::new(
                &aws_config::load_defaults(BehaviorVersion::latest()).await,
            )),
            config,
        }
    }
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for AWSHealthCheckHandler {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        /* maybe we can grab this from a central location instead of the struct itself? cache? */
        let client = self.lambda_client.clone();
        Box::pin(async move {
            if self.config.enabled {
                let mut response = ApiGatewayProxyResponse::default();
                let response_status: u32 = if self.config.downstream_enabled {
                    let payload = Blob::new(self.config.downstream_function_health_payload.clone());
                    let function_name = self.config.downstream_function.clone();
                    match client.unwrap().invoke()
                        .function_name(&function_name)
                        .payload(payload)
                        .send().await {
                        Ok(response) => {
                            response.status_code as u32
                        }
                        Err(_) => 503u32
                    }
                } else {
                    HEALTH_STATUS
                };

                response.headers.insert(CONTENT_TYPE, "plain/text".parse().unwrap());
                if response_status.gt(&200u32) && response_status.lt(&300u32) {
                    response.body = Some(HEALTH_BODY.into());
                    response.status_code = HEALTH_STATUS as i64
                } else {
                    response.status_code = response_status as i64;
                    response.body = Some(HEALTH_ERROR.into());
                }
                exchange.save_output(response);
                Ok(())
            } else {
                Err(())
            }
        })
    }
}
