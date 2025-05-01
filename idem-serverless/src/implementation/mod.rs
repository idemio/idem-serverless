pub mod cors;
pub mod echo;
pub mod header;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod traceability;

use crate::implementation::jwt::handler::JwtValidationHandler;
use crate::implementation::{
    cors::handler::CorsHandler, header::handler::HeaderHandler,
    health::handler::HealthCheckHandler, proxy::handler::LambdaProxyHandler,
    traceability::handler::TraceabilityHandler,
};
use idem_config::config::{
    Config, ConfigProvider, DefaultConfigProvider, FileConfigProvider, ProviderType,
};
use idem_handler::exchange::Exchange;
use idem_handler::factory::HandlerFactory;
use idem_handler::handler::Handler;
use idem_handler::status::{HandlerExecutionError, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
pub type HandlerOutput<'a> =
    Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'a>>;

pub enum LambdaHandler {
    ProxyHandler(LambdaProxyHandler),
    CorsHandler(CorsHandler),
    HeaderHandler(HeaderHandler),
    TraceabilityHandler(TraceabilityHandler),
    HealthCheckHandler(HealthCheckHandler),
    JwtValidationHandler(JwtValidationHandler),
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for LambdaHandler {
    fn exec<'i1, 'i2, 'o>(&'i1 self, exchange: &'i2 mut LambdaExchange) -> HandlerOutput<'o>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        match self {
            LambdaHandler::ProxyHandler(handler) => handler.exec(exchange),
            LambdaHandler::CorsHandler(handler) => handler.exec(exchange),
            LambdaHandler::HeaderHandler(handler) => handler.exec(exchange),
            LambdaHandler::TraceabilityHandler(handler) => handler.exec(exchange),
            LambdaHandler::HealthCheckHandler(handler) => handler.exec(exchange),
            LambdaHandler::JwtValidationHandler(handler) => handler.exec(exchange),
        }
    }
}

pub struct LambdaHandlerFactory;

impl HandlerFactory<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>
    for LambdaHandlerFactory
{
    type Err = ();
    type CreatedHandler = LambdaHandler;

    fn create_handler(
        name: &str,
        provider_type: ProviderType,
    ) -> Result<Self::CreatedHandler, Self::Err> {
        match name {
            "ProxyHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "proxy.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::ProxyHandler(LambdaProxyHandler::new(config)))
            }
            "TraceabilityHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "trace.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::TraceabilityHandler(
                    TraceabilityHandler::new(config),
                ))
            }
            "HeaderHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "header.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::HeaderHandler(HeaderHandler::new(config)))
            }
            "JwtValidationHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "jwt_validator.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::JwtValidationHandler(
                    JwtValidationHandler::new(config),
                ))
            }
            "CorsHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "cors.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::CorsHandler(CorsHandler::new(config)))
            }
            "HealthHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "health.json".into(),
                        base_path: "/opt/config".into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::HealthCheckHandler(HealthCheckHandler::new(
                    config,
                )))
            }
            _ => Err(()),
        }
    }
}

pub struct LambdaHandlerExecutor {
    pub handlers: Vec<LambdaHandler>,
}

impl LambdaHandlerExecutor {
    pub fn new(handlers: Vec<LambdaHandler>) -> LambdaHandlerExecutor {
        LambdaHandlerExecutor { handlers }
    }
}
