use crate::exchange::Exchange;
use crate::executor::HandlerExecutionError;
use crate::handlers::cors_handler::{CorsHandler, CorsHandlerConfig};
use crate::handlers::header_handler::{HeaderHandler, HeaderHandlerConfig};
use crate::handlers::health_check_handler::{HealthCheckHandler, HealthCheckHandlerConfig};
use crate::handlers::invoke_lambda_handler::{LambdaProxyHandler, LambdaProxyHandlerConfig};
use crate::handlers::sanitizer_handler::SanitizerHandler;
use crate::handlers::specification_handler::SpecificationHandler;
use crate::handlers::traceability_handler::{TraceabilityHandler, TraceabilityHandlerConfig};
use crate::status::HandlerStatus;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::string::ToString;

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

pub trait Handler<I, O, M>: Send
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{
    type Err;
    type Status;

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<I, O, M>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Status, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}

pub trait AsyncFromStr: Send
where
    Self: Sized,
{
    type Err;
    fn async_from_str<'i1, 'o>(
        s: &'i1 str,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        Self: 'o;
}

pub enum HandlerRegister {
    ProxyHandler(LambdaProxyHandler),
    CorsHandler(CorsHandler),
    HeaderHandler(HeaderHandler),
    TraceabilityHandler(TraceabilityHandler),
    HealthCheckHandler(HealthCheckHandler),
    SanitizerHandler(SanitizerHandler),
    SpecificationHandler(SpecificationHandler),
    Custom(
        Box<
            dyn Handler<
                ApiGatewayProxyRequest,
                ApiGatewayProxyResponse,
                Context,
                Err = HandlerExecutionError,
                Status = HandlerStatus,
            >,
        >,
    ),
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

impl AsyncFromStr for HandlerRegister {
    type Err = InvalidHandlerError;

    fn async_from_str<'i1, 'o>(
        s: &'i1 str,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        Self: 'o,
    {
        Box::pin(async move {
            // TODO - handle loading configurations here instead of using default...
            match s.to_lowercase().as_str() {
                "idem.proxyhandler" => Ok(HandlerRegister::ProxyHandler(
                    LambdaProxyHandler::new(LambdaProxyHandlerConfig::default()).await,
                )),

                "idem.corshandler" => Ok(HandlerRegister::CorsHandler(
                    CorsHandler::new(CorsHandlerConfig::default()).await,
                )),

                "idem.headerhandler" => Ok(HandlerRegister::HeaderHandler(
                    HeaderHandler::new(HeaderHandlerConfig::default()).await,
                )),

                "idem.traceabilityhandler" => Ok(HandlerRegister::TraceabilityHandler(
                    TraceabilityHandler::new(TraceabilityHandlerConfig::default()).await,
                )),

                "idem.healthcheck" => Ok(HandlerRegister::HealthCheckHandler(
                    HealthCheckHandler::new(HealthCheckHandlerConfig::default()).await,
                )),

                _ => Err(InvalidHandlerError {
                    handler_name: s.to_owned(),
                }),
            }
        })
    }
}
