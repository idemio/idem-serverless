use std::future::Future;
use std::pin::Pin;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Body, Context};
use idem_config::config::Config;
use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use crate::implementation::echo::config::EchoRequestHandlerConfig;

pub struct EchoRequestHandler {
    config: Config<EchoRequestHandlerConfig>
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>
for EchoRequestHandler
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
            if !self.config.get().enabled {
                return Ok(HandlerStatus::new(Code::DISABLED))
            }

            let request_payload = exchange.consume_request().unwrap();
            let echo_body: Option<Body> = if self.config.get().static_body.is_some() {
                match self.config.get().static_body.as_ref() {
                    Some(x) if !x.is_empty() => Some(Body::Text(x.clone())),
                    Some(_) => None,
                    None => None,
                }
            } else {
                match request_payload.body {
                    Some(body) => Some(Body::Text(body)),
                    None => None
                }
            };

            let mut response_payload = ApiGatewayProxyResponse {
                status_code: 200,
                body: echo_body,
                ..Default::default()
            };

            if self.config.get().echo_headers {
                let request_headers = request_payload.headers;
                response_payload.headers.extend(request_headers);
            }

            exchange.save_output(response_payload);
            Ok(HandlerStatus::new(Code::OK))
        })
    }
}