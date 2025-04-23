use crate::exchange::Exchange;
use crate::handlers::Handler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use crate::executor::HandlerExecutionError;
use crate::status::HandlerStatus;

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct SpecificationHandlerConfig {
    enabled: bool,
    multiple_spec: bool,
    ignore_invalid_path: bool,
    path_spec_mapping: HashMap<String, String>,
}

pub(crate) struct SpecificationHandler {
    config: SpecificationHandlerConfig,
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for SpecificationHandler {
    type Err = HandlerExecutionError;
    type Status = HandlerStatus;

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Status, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        Box::pin(async move { todo!("implement specification handler...") })
    }
}
