use crate::implementation::cors_handler::{CorsHandler, CorsHandlerConfig};
use crate::implementation::header_handler::{HeaderHandler, HeaderHandlerConfig};
use crate::implementation::health_check_handler::{HealthCheckHandler, HealthCheckHandlerConfig};
use crate::implementation::invoke_lambda_handler::{LambdaProxyHandler, LambdaProxyHandlerConfig};
use crate::implementation::sanitizer_handler::SanitizerHandler;
use crate::implementation::specification_handler::SpecificationHandler;
use crate::implementation::traceability_handler::{TraceabilityHandler, TraceabilityHandlerConfig};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::string::ToString;
use idem_handler::handler::{Handler, HandlerLoader};
use crate::entry::LambdaHandler;

mod basic_auth_handler;
mod body_transform_handler;
mod cors_handler;
mod echo_test_middleware;
mod header_handler;
mod health_check_handler;
mod invoke_lambda_handler;
mod jwt_verify_handler;
mod limit_handler;
mod metrics_handler;
mod payload_validation_handler;
mod router_handler;
mod sanitizer_handler;
mod specification_handler;
mod swt_verify_handler;
mod traceability_handler;
mod info_handler;

pub enum HandlerRegister {
    ProxyHandler(LambdaProxyHandler),
    CorsHandler(CorsHandler),
    HeaderHandler(HeaderHandler),
    TraceabilityHandler(TraceabilityHandler),
    HealthCheckHandler(HealthCheckHandler),
    SanitizerHandler(SanitizerHandler),
    SpecificationHandler(SpecificationHandler),
    Custom(LambdaHandler),
}

#[derive(Debug)]
pub struct InvalidHandlerError {
    handler_name: String,
}

impl Display for InvalidHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[InvalidHandlerError] Invalid handler name: {}",
            self.handler_name
        )
    }
}

impl std::error::Error for InvalidHandlerError {}

impl HandlerLoader<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for HandlerRegister {
    type Err = InvalidHandlerError;

    fn async_from_str<'i1, 'o>(
        s: &'i1 str,
    ) -> Pin<Box<dyn Future<Output = Result<LambdaHandler, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        Self: 'o,
    {
        Box::pin(async move {
            // TODO - handle loading configurations here instead of using default...
            let res: Result<LambdaHandler, Self::Err> = match s.to_lowercase().as_str() {
                "idem.proxyhandler" => Ok(Box::new(
                    LambdaProxyHandler::new(LambdaProxyHandlerConfig {
                        enabled: true,
                        functions: HashMap::from([("/path/to/resource@POST".to_string(), "arn:aws:lambda:ca-central-1:173982495217:function:test-lambda-function-destination".to_string())]),
                        ..Default::default()
                    }).await,
                )),

                "idem.corshandler" => Ok(Box::new(
                    CorsHandler::new(CorsHandlerConfig::default()).await,
                )),

                "idem.headerhandler" => Ok(Box::new(
                    HeaderHandler::new(HeaderHandlerConfig::default()).await,
                )),

                "idem.traceabilityhandler" => Ok(Box::new(
                    TraceabilityHandler::new(TraceabilityHandlerConfig {
                        enabled: true,
                        autogen_correlation_id: true,
                        traceability_header_name: "x-traceability-id".to_string(),
                        correlation_header_name: "x-correlation-id".to_string(),
                        add_trace_to_response: true,
                        ..Default::default()
                    }).await,
                )),

                "idem.healthcheck" => Ok(Box::new(
                    HealthCheckHandler::new(HealthCheckHandlerConfig::default()).await,
                )),

                _ => Err(InvalidHandlerError {
                    handler_name: s.to_owned(),
                }),
            };
            res
        })
    }
}
