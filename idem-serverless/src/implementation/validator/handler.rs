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
use std::collections::HashMap;

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
            ));

            if self.config.get().validate_request {
                if let Some(path) = &exchange.input().unwrap().path {
                    let converted_headers = &exchange
                        .input()
                        .unwrap()
                        .headers
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
                        .collect::<HashMap<String, String>>();

                    let converted_query_params = &exchange
                        .input()
                        .unwrap()
                        .query_string_parameters
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<HashMap<String, String>>();

                    let method = &exchange.input().unwrap().http_method.as_str();
                    let json_body = serde_json::to_value(converted_headers).unwrap();

                    if validator
                        .validate_request(
                            path,
                            method,
                            Some(&json_body),
                            Some(&exchange
                                .input()
                                .unwrap()
                                .headers),
                            Some(&converted_query_params),
                        )
                        .is_err()
                    {
                        return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                            .set_message("Request validation failed"));
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
