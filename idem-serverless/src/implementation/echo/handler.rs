use std::future::Future;
use std::pin::Pin;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};

#[derive(Clone, Default)]
pub(crate) struct EchoTestLambdaMiddleware;

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>
for EchoTestLambdaMiddleware
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<
            ApiGatewayProxyRequest,
            ApiGatewayProxyResponse,
            Context,
        >,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'o>>
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
            Ok(HandlerStatus::new(Code::OK))
        })
    }
}