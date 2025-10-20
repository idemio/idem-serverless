pub mod cors;
pub mod echo;
pub mod header;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod traceability;
mod validator;
mod sanitizer;
use idemio::exchange::Exchange;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
