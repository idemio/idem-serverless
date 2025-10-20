use core::clone::Clone;
use core::option::Option::Some;
use core::prelude::rust_2024::Ok;
use core::result::Result;
use core::todo;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::handler::LambdaExchange;
use idemio::config::ProviderType;
use idemio::exchange::Exchange;
use idemio::handler::Handler;
use idemio::router::executor::DefaultExecutor;
use idemio::router::factory::{ExchangeFactory, ExchangeFactoryError};
use idemio::router::path::http::HttpPathMethodMatcher;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};

pub struct LambdaExchangeFactory;

#[async_trait]
impl ExchangeFactory<ApiGatewayProxyRequest, Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayProxyRequestContext>>
for LambdaExchangeFactory
{
    async fn extract_route_info<'a>(
        &self,
        request: &'a ApiGatewayProxyRequest,
    ) -> Result<(&'a str, &'a str), ExchangeFactoryError> {
        let method = request.http_method.as_str();
        let path = request.path.as_ref().unwrap();
        Ok((path, method))
    }

    async fn create_exchange<'req>(
        &self,
        request: ApiGatewayProxyRequest,
    ) -> Result<
        Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayProxyRequestContext>,
        ExchangeFactoryError,
    > {
        let mut exchange = Exchange::new();
        let metadata = request.request_context.clone();
        exchange.set_input(request);
        exchange.set_metadata(metadata);
        Ok(exchange)
    }
}

type LambdaRouter = idemio::router::RequestRouter<
    ApiGatewayProxyRequest,
    Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayProxyRequestContext>,
    LambdaExchangeFactory,
    DefaultExecutor<ApiGatewayProxyRequest>,
    HttpPathMethodMatcher<
        Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayProxyRequestContext>
    >,
>;

pub async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    todo!()
}
