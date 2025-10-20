use std::convert::Infallible;
use crate::ROOT_CONFIG_PATH;
use crate::handler::LambdaExchange;
use async_trait::async_trait;
use http::{HeaderMap, Method, Request};
use idemio::config::Config;
use idemio::exchange::Exchange;
use idemio::handler::Handler;
use idemio::status::{ExchangeState, HandlerStatus};
//use idem_handler::handler::Handler;
//use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
//use idem_handler_config::config::Config;
//use idem_handler_macro::ConfigurableHandler;
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use oasert::types::HttpLike;
use oasert::validator::OpenApiPayloadValidator;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct ValidatorHandlerConfig {
    pub enable: bool,
    pub validate_request: bool,
    pub validate_response: bool,
    pub openapi_specification: String,

    #[serde(skip)]
    loaded_openapi_specification: Option<OpenApiPayloadValidator>,
}

impl Default for ValidatorHandlerConfig {
    fn default() -> Self {
        let spec = std::fs::read_to_string(format!("{}/openapi.json", ROOT_CONFIG_PATH)).expect("Unable to read openapi.json file");
        let spec: Value = serde_json::from_str(&spec).expect("Unable to parse openapi.json file");
        let validator = OpenApiPayloadValidator::new(spec).expect("Unable to create validator from openapi.json file");
        
        Self {
            enable: true,
            validate_request: true,
            validate_response: false,
            openapi_specification: "openapi.json".to_string(),
            loaded_openapi_specification: Some(validator),
        }
    }
}


//#[derive(ConfigurableHandler)]
pub struct ValidatorHandler {
    config: Config<ValidatorHandlerConfig>,
}

#[async_trait]
impl Handler<Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>> for ValidatorHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, Infallible> {
        if !self.config.get().enable {
            return Ok(HandlerStatus::new(ExchangeState::DISABLED));
        }

        if self.config.get().loaded_openapi_specification.is_some() {
            let validator = self.config.get().loaded_openapi_specification.as_ref().unwrap();
            let request = exchange.input().await.unwrap();
            let request = ApiGatewayProxyRequestWrapper::new(request);
            let result = validator.validate_request(&request, None);
            if result.is_err() {
                return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                    .message("Request validation failed"));
            }
        }


        Ok(HandlerStatus::new(ExchangeState::OK))
    }

    fn name(&self) -> &str {
        "ValidatorHandler"
    }
}

struct ApiGatewayProxyRequestWrapper<'a> {
    request: &'a ApiGatewayProxyRequest,
    body: Option<Value>,
    query_params: Option<String>,
    path: String,
}

impl<'a> ApiGatewayProxyRequestWrapper<'a> {
    pub fn new(request: &'a ApiGatewayProxyRequest) -> Self {
        let path = request.path.clone().unwrap_or("/".to_string());
        let query_params: Option<String> = if !request.query_string_parameters.is_empty() {
            Some(request.query_string_parameters.to_query_string())   
        } else {
            None
        };
        
        let body: Option<Value> = match request.body.as_ref() {
            None => None,
            Some(found) => {
                match serde_json::from_str(found) {
                    Ok(x) => Some(x),
                    Err(_) => None,
                }
            }
        };
        
        Self {
            request,
            body,
            query_params,
            path,
        }
    }
}



impl HttpLike<String> for ApiGatewayProxyRequestWrapper<'_>
{
    fn method(&self) -> &Method {
        &self.request.http_method
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn headers(&self) -> &HeaderMap {
        &self.request.headers
    }

    fn body(&self) -> Option<Value> {
        // TODO - change to ref of value to reduce clone.
        match &self.body {
            None => None,
            Some(body) => Some(body.clone())
        }
    }

    fn query(&self) -> Option<&str> {
        match &self.query_params {
            None => None,
            Some(query_params) => Some(query_params.as_str())
        }
    }
}
