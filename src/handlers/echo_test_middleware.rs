use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use crate::exchange::{Exchange, Handler};

#[derive(Debug, Clone, Default)]
pub(crate) struct EchoTestLambdaMiddleware;

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>> for EchoTestLambdaMiddleware {
    fn process<'i1, 'i2, 'o>(&'i1 self, context: &'i2 mut Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>>) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where 'i1: 'o, 'i2: 'o, Self: 'o
    {
        Box::pin(async move {
            let request_payload = context.consume_request().unwrap();
            let response_payload = ApiGatewayProxyResponse {
                status_code: 200,
                body: Some(request_payload.body.unwrap_or_default().into()),
                ..Default::default()
            };
            context.save_output(response_payload);
            Ok(())
        })
    }
}