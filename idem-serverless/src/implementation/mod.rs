pub mod cors;
pub mod echo;
pub mod header;
pub mod health;
pub mod proxy;
pub mod traceability;
mod jwt;

use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::{HandlerExecutionError, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::future::Future;
use std::pin::Pin;

use crate::implementation::{
    cors::handler::CorsHandler, header::handler::HeaderHandler,
    health::handler::HealthCheckHandler, proxy::handler::LambdaProxyHandler,
    traceability::handler::TraceabilityHandler,
};

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
pub type HandlerOutput<'a> =
Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'a>>;

pub enum LambdaHandlers {
    ProxyHandler(LambdaProxyHandler),
    CorsHandler(CorsHandler),
    HeaderHandler(HeaderHandler),
    TraceabilityHandler(TraceabilityHandler),
    HealthCheckHandler(HealthCheckHandler),
    //    SanitizerHandler(SanitizerHandler),
    //SpecificationHandler(SpecificationHandler)
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for LambdaHandlers {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut LambdaExchange,
    ) -> HandlerOutput<'o>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        match self {
            LambdaHandlers::ProxyHandler(handler) => handler.process(exchange),
            LambdaHandlers::CorsHandler(handler) => handler.process(exchange),
            LambdaHandlers::HeaderHandler(handler) => handler.process(exchange),
            LambdaHandlers::TraceabilityHandler(handler) => handler.process(exchange),
            LambdaHandlers::HealthCheckHandler(handler) => handler.process(exchange),
            //            LambdaHandlers::SanitizerHandler(handler) => handler.process(exchange),
            //LambdaHandlers::SpecificationHandler(handler) => handler.process(exchange),
        }
    }
}

pub struct LambdaHandlerExecutor {
    pub handlers: Vec<LambdaHandlers>,
}

impl LambdaHandlerExecutor {
    pub fn new(handlers: Vec<LambdaHandlers>) -> LambdaHandlerExecutor {
        LambdaHandlerExecutor { handlers }
    }
}
