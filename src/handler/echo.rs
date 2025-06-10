use serde::Deserialize;
use async_trait::async_trait;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Body, Context};
use crate::handler::LambdaExchange;

#[derive(Default, Deserialize)]
pub struct EchoRequestHandlerConfig {
    pub enabled: bool,
    pub echo_headers: bool,
    pub static_body: Option<String>
}



#[derive(ConfigurableHandler)]
pub struct EchoRequestHandler {
    config: Config<EchoRequestHandlerConfig>,
}

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for EchoRequestHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError> {
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        let request_payload = exchange.take_request().unwrap();
        let echo_body: Option<Body> = if self.config.get().static_body.is_some() {
            match self.config.get().static_body.as_ref() {
                Some(x) if !x.is_empty() => Some(Body::Text(x.clone())),
                Some(_) => None,
                None => None,
            }
        } else {
            match request_payload.body {
                Some(body) => Some(Body::Text(body)),
                None => None,
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
    }
}

