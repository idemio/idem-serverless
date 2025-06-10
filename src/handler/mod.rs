pub mod cors;
pub mod echo;
pub mod header;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod traceability;
mod validator;
mod sanitizer;

use core::convert::Into;
use core::prelude::rust_2024::Ok;
use core::result::Result;
use async_trait::async_trait;
use idem_handler::exchange::Exchange;
use idem_handler::factory::HandlerFactory;
use idem_handler::handler::Handler;
use idem_handler::status::{HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::{Config, DefaultConfigProvider, FileConfigProvider, ProviderType};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use crate::handler::cors::CorsHandler;
use crate::handler::header::HeaderHandler;
use crate::handler::health::HealthCheckHandler;
use crate::handler::jwt::JwtValidationHandler;
use crate::handler::proxy::LambdaProxyHandler;
use crate::handler::traceability::TraceabilityHandler;
use crate::ROOT_CONFIG_PATH;

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;

pub enum LambdaHandler {
    ProxyHandler(LambdaProxyHandler),
    CorsHandler(CorsHandler),
    HeaderHandler(HeaderHandler),
    TraceabilityHandler(TraceabilityHandler),
    HealthCheckHandler(HealthCheckHandler),
    JwtValidationHandler(JwtValidationHandler),
}

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for LambdaHandler {
    async fn exec(
        &self,
        exchange: & mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError>
    {
        match self {
            LambdaHandler::ProxyHandler(handler) => handler.exec(exchange).await,
            LambdaHandler::CorsHandler(handler) => handler.exec(exchange).await,
            LambdaHandler::HeaderHandler(handler) => handler.exec(exchange).await,
            LambdaHandler::TraceabilityHandler(handler) => handler.exec(exchange).await,
            LambdaHandler::HealthCheckHandler(handler) => handler.exec(exchange).await,
            LambdaHandler::JwtValidationHandler(handler) => handler.exec(exchange).await,
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
