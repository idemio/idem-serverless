use crate::exchange::Exchange;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use crate::executor::HandlerExecutionError;
use crate::handlers::Handler;
use crate::status::{HandlerStatus, HandlerStatusCode};

#[derive(Clone, Default)]
pub(crate) struct EchoTestLambdaMiddleware;

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>>
    for EchoTestLambdaMiddleware
{
    type Err = HandlerExecutionError;
    type Status = HandlerStatus;

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<
            ApiGatewayProxyRequest,
            ApiGatewayProxyResponse,
            HashMap<String, String>,
        >,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Status, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        Box::pin(async move {
            let request_payload = exchange.consume_request().unwrap();
            let response_payload = ApiGatewayProxyResponse {
                status_code: 200,
                body: Some(request_payload.body.unwrap_or_default().into()),
                ..Default::default()
            };
            exchange.save_output(response_payload);
            Ok(HandlerStatus::from(HandlerStatusCode::Ok))
        })
    }
}
