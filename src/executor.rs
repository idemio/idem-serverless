use crate::exchange::{AttachmentKey, Exchange};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use crate::config::config::LoadMethod;
use crate::handlers::Handler;
use crate::handlers::invoke_lambda_handler::{AWSLambdaFunctionProxyHandler, AWSLambdaFunctionProxyHandlerConfig};

pub const LAMBDA_CONTEXT: AttachmentKey = AttachmentKey(4);
pub(crate) const LOAD_METHOD: &str = "LOAD_METHOD";
const CACHE_CONFIGS: &str = "CACHE_CONFIGS";


fn load_handlers() -> Vec<Box<dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>>> {
    let ev_load_method = match match env::var(LOAD_METHOD) {
        Ok(v) => {
            LoadMethod::from_str(v.as_str())
        },
        Err(_) => Ok(LoadMethod::Default)
    } {
        Ok(method) => method,
        Err(_) => todo!("Handle invalid load method from env variables.")
    };

    let ev_cache_configs: bool = match env::var(CACHE_CONFIGS) {
        Ok(v) => v.parse::<bool>().unwrap_or_default(),
        Err(_) => false
    };

    todo!()
}

pub(crate) async fn entry(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {


    let proxy_handler = AWSLambdaFunctionProxyHandler::new(AWSLambdaFunctionProxyHandlerConfig::default()).await;


    let middleware: Vec<Box<dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> + Send>> = vec![Box::new(proxy_handler)];
    let mut executor = LambdaMiddlewareExecutor::new(middleware);
    let (payload, context) = event.into_parts();
    let mut exchange: Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> = Exchange::new();
    exchange.save_input(payload);
    for middleware in &executor.middlewares {
        match middleware.process(&mut exchange).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
    Ok(exchange.consume_output().unwrap())
}

pub struct LambdaMiddlewareExecutor {
    middlewares: Vec<
        Box<
            dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>
                + Send,
        >,
    >,
}

impl LambdaMiddlewareExecutor {
    pub fn new(
        middleware: Vec<
            Box<
                dyn Handler<
                        ApiGatewayProxyRequest,
                        ApiGatewayProxyResponse,
                        Context,
                    > + Send,
            >,
        >,
    ) -> Self {
        Self {
            middlewares: middleware,
        }
    }
}
