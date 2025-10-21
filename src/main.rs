use async_trait::async_trait;
use core::result::Result;
use idemio::config::{Config, DefaultConfigProvider};
use idemio::exchange::Exchange;
use idemio::handler::registry::HandlerRegistry;
use idemio::handler::HandlerId;
use idemio::router::config::builder::{
    MethodBuilder, RouteBuilder, ServiceBuilder, SingleServiceConfigBuilder,
};
use idemio::router::executor::DefaultExecutor;
use idemio::router::factory::{ExchangeFactory, ExchangeFactoryError, RouteInfo};
use idemio::router::path::http::HttpPathMethodMatcher;
use idemio::router::path::PathMatcher;
use idemio::router::{RequestRouter, Router, RouterBuilder};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::tracing::init_default_subscriber;
use lambda_http::{lambda_runtime, service_fn, Body, Context, Error, LambdaEvent};
use std::marker::PhantomData;
use std::sync::Arc;

pub(crate) mod handler;

use crate::handler::header::HeaderHandler;
use crate::handler::jwt::JwtValidationHandler;
use crate::handler::proxy::LambdaProxyHandler;

pub const ROOT_CONFIG_PATH: &str = "/opt/config";

type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
type LambdaPathRouter = HttpPathMethodMatcher<LambdaExchange>;
type IncomingLambdaRequest = ApiGatewayProxyRequest;
type OutgoingLambdaResponse = ApiGatewayProxyResponse;
struct LambdaExchangeFactory;

#[async_trait]
impl ExchangeFactory<IncomingLambdaRequest, LambdaExchange> for LambdaExchangeFactory {
    async fn extract_route_info<'a>(
        &self,
        request: &'a IncomingLambdaRequest,
    ) -> Result<RouteInfo<'a>, ExchangeFactoryError> {
        let path = match request.path.as_ref() {
            None => None,
            Some(val) => Some(val.as_str()),
        };
        let method = Some(request.http_method.as_str());
        Ok(RouteInfo { path, method })
    }

    async fn create_exchange<'req>(
        &self,
        request: IncomingLambdaRequest,
    ) -> Result<LambdaExchange, ExchangeFactoryError> {
        let mut exchange = Exchange::new();
        exchange.set_input(request);
        Ok(exchange)
    }
}

type AwsLambdaRouter = RequestRouter<
    IncomingLambdaRequest,
    LambdaExchange,
    LambdaExchangeFactory,
    DefaultExecutor<OutgoingLambdaResponse>,
    LambdaPathRouter,
>;

// TODO - these will be changed to be configurable, for now we just use the default config for all handlers and statically set our endpoints.
fn create_router() -> AwsLambdaRouter {
    let mut handler_registry = HandlerRegistry::new();
    // only use the header handler, jwt handler, and proxy handler for now.
    let header_handler = HeaderHandler {
        config: Config::new(DefaultConfigProvider).unwrap(),
    };
    handler_registry
        .register_handler(HandlerId::new("HeaderHandler"), header_handler)
        .unwrap();
    let jwt_handler = JwtValidationHandler {
        config: Config::new(DefaultConfigProvider).unwrap(),
    };
    handler_registry
        .register_handler(HandlerId::new("JwtValidationHandler"), jwt_handler)
        .unwrap();
    let proxy_handler = LambdaProxyHandler {
        config: Config::new(DefaultConfigProvider).unwrap(),
    };
    handler_registry
        .register_handler(HandlerId::new("LambdaProxyHandler"), proxy_handler)
        .unwrap();
    let router_config = SingleServiceConfigBuilder::new()
        .route("/test")
        .get()
        .request_handler("JwtValidationHandler")
        .request_handler("HeaderHandler")
        .termination_handler("LambdaProxyHandler")
        .end_method()
        .end_route()
        .build();

    let matcher = HttpPathMethodMatcher::new(&router_config, &handler_registry).unwrap();
    let executor: DefaultExecutor<OutgoingLambdaResponse> = DefaultExecutor {
        _phantom: PhantomData::default(),
    };
    let factory = LambdaExchangeFactory;
    RouterBuilder::new()
        .factory(factory)
        .executor(executor)
        .matcher(matcher)
        .build()
}

async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
    router: Arc<AwsLambdaRouter>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let request = event.payload;
    let context = event.context;
    match router.route(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            let mut response = ApiGatewayProxyResponse::default();
            response.body = Some(Body::Text(format!("Error: {}", e)));
            Ok(response)
        }
    }
}

fn main() -> Result<(), Error> {
    let router = Arc::new(create_router());
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            init_default_subscriber();
            lambda_runtime::run(service_fn(|event| entry(event, router.clone()))).await
        })
}
