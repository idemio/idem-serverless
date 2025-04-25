use crate::implementation::Handler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use lambda_http::Context;
use idem_handler::status::{HandlerExecutionError, HandlerStatus};
use crate::entry::LambdaExchange;

#[derive(Deserialize, Serialize, Clone, Default)]
pub(crate) struct SanitizerHandlerConfig {
    enabled: bool,
    body_enabled: bool,
    body_encoder: SanitizerEncoder,
    body_options: SanitizerSectionConfig,
    header_options: SanitizerSectionConfig,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub(crate) struct SanitizerSectionConfig {
    attributes_to_encode: Vec<String>,
    attributes_to_ignore: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
enum SanitizerEncoder {
    #[default]
    JavaScriptSource,
}

pub(crate) struct SanitizerHandler {
    config: SanitizerHandlerConfig,
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for SanitizerHandlerConfig {

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut LambdaExchange,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        Box::pin(async move { todo!("Implement sanitizer handler...") })
    }
}
