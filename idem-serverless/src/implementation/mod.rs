pub mod cors;
pub mod echo;
pub mod header;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod traceability;
mod validator;

use crate::implementation::jwt::handler::JwtValidationHandler;
use crate::implementation::{
    cors::handler::CorsHandler, header::handler::HeaderHandler,
    health::handler::HealthCheckHandler, proxy::handler::LambdaProxyHandler,
    traceability::handler::TraceabilityHandler,
};
use idem_config::config::{
    Config, DefaultConfigProvider, FileConfigProvider, ProviderType,
};
use idem_handler::exchange::Exchange;
use idem_handler::factory::HandlerFactory;
use idem_handler::handler::Handler;
use idem_handler::status::{HandlerExecutionError, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::future::Future;
use std::pin::Pin;
use crate::ROOT_CONFIG_PATH;

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
    fn exec<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut LambdaExchange,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result,
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
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::ProxyHandler(
                    LambdaProxyHandler::init_handler(config),
                ))
            }
            "TraceabilityHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "trace.json".into(),
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::TraceabilityHandler(
                    TraceabilityHandler::init_handler(config),
                ))
            }
            "HeaderHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "header.json".into(),
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::HeaderHandler(HeaderHandler::init_handler(
                    config,
                )))
            }
            "JwtValidationHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "jwt_validator.json".into(),
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::JwtValidationHandler(
                    JwtValidationHandler::init_handler(config),
                ))
            }
            "CorsHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "cors.json".into(),
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::CorsHandler(CorsHandler::init_handler(
                    config,
                )))
            }
            "HealthHandler" => {
                let config = match provider_type {
                    ProviderType::File => Config::new(FileConfigProvider {
                        config_name: "health.json".into(),
                        base_path: ROOT_CONFIG_PATH.into(),
                    }),
                    ProviderType::Default => Config::new(DefaultConfigProvider),
                }?;
                Ok(LambdaHandler::HealthCheckHandler(
                    HealthCheckHandler::init_handler(config),
                ))
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
