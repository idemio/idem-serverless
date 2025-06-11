use crate::ROOT_CONFIG_PATH;
use crate::handler::LambdaExchange;
use async_trait::async_trait;
use http::Request;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ValidatorHandlerConfig {
    pub enable: bool,
    pub validate_request: bool,
    pub validate_response: bool,
    pub openapi_specification: String,
}

impl Default for ValidatorHandlerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            validate_request: true,
            validate_response: false,
            openapi_specification: "openapi.json".to_string(),
        }
    }
}

#[derive(ConfigurableHandler)]
pub struct ValidatorHandler {
    config: Config<ValidatorHandlerConfig>,
}

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for ValidatorHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError> {
        if !self.config.get().enable {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        let request = exchange.input().unwrap();
        let request_uri: String = format!(
            "{}{}",
            request.path.as_ref().unwrap_or(&String::from("/")),
            request
                .query_string_parameters
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&")
        );
        let request = Request::builder()
            .method(&request.http_method)
            .uri(&request_uri)
            .body(request.body.as_ref().unwrap_or(&String::from("")))
            .unwrap();
        
        todo!()

        //
        //        let spec =
        //        let validator = oasert::validator::OpenApiPayloadValidator::new();
        //
        //        if self.config.get().validate_request {
        //            if let Some(path) = &exchange.input().unwrap().path {
        //                let converted_headers = &exchange
        //                    .input()
        //                    .unwrap()
        //                    .headers
        //                    .iter()
        //                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        //                    .collect::<HashMap<String, String>>();
        //
        //                let converted_query_params = &exchange
        //                    .input()
        //                    .unwrap()
        //                    .query_string_parameters
        //                    .iter()
        //                    .map(|(k, v)| (k.to_string(), v.to_string()))
        //                    .collect::<HashMap<String, String>>();
        //
        //                let method = &exchange.input().unwrap().http_method.as_str();
        //                let json_body = serde_json::to_value(converted_headers).unwrap();
        //
        //                if validator
        //                    .validate_request(
        //                        path,
        //                        method,
        //                        Some(&json_body),
        //                        Some(converted_headers),
        //                        Some(&converted_query_params),
        //                    )
        //                    .is_err()
        //                {
        //                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
        //                        .set_message("Request validation failed"));
        //                }
        //            }
        //        }
        //
        //        if self.config.get().validate_response {
        //            // TODO -  attach validator on response callback
        //        }
        //
        //        Ok(HandlerStatus::new(Code::OK))
    }
}
