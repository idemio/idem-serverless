use std::collections::HashMap;
use crate::implementation::validator::config::ValidatorHandlerConfig;
use crate::implementation::LambdaExchange;
use crate::ROOT_CONFIG_PATH;
use idem_config::config::Config;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerStatus};
use idem_handler::HandlerOutput;
use idem_macro::ConfigurableHandler;
use idem_openapi::OpenApiValidator;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;

#[derive(ConfigurableHandler)]
pub struct ValidatorHandler {
    config: Config<ValidatorHandlerConfig>,
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for ValidatorHandler {
    fn exec<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut LambdaExchange,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result,
    {
        Box::pin(async move {
            if !self.config.get().enable {
                return Ok(HandlerStatus::new(Code::DISABLED));
            }

            let validator = OpenApiValidator::from_file(&format!(
                "{}/{}",
                ROOT_CONFIG_PATH,
                self.config.get().openapi_specification
            ))
            .unwrap();

            if self.config.get().validate_request {
                if let Some(path) = &exchange.input().unwrap().path {
                    let in_headers = &exchange.input().unwrap().headers;
                    let mut out_headers: HashMap<String, String> = HashMap::new();
                    in_headers.iter().for_each(|(k, v)| {
                        out_headers.insert(k.to_string(), v.to_str().unwrap().to_string());
                    });

                    let in_query_params = &exchange.input().unwrap().query_string_parameters;
                    let mut out_query_params: HashMap<String, String> = HashMap::new();
                    in_query_params.iter().for_each(|(k, v)| {
                        out_query_params.insert(k.to_string(), v.to_string());
                    });

                    let method = &exchange.input().unwrap().http_method.as_str();
                    let json_body = serde_json::to_value(exchange.input().unwrap()).unwrap();

                    if validator.validate_request(path, method, Some(&out_headers), Some(&out_query_params), Some(&json_body)).is_err() {
                        return Ok(HandlerStatus::new(Code::CLIENT_ERROR).set_message(
                            "Request validation failed"))
                    }

                }

            }

            if self.config.get().validate_response {
                // TODO -  attach validator on response callback
            }

            Ok(HandlerStatus::new(Code::OK))
        })
    }
}
