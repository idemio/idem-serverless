use crate::config::config::LoadMethod;
use crate::exchange::{AttachmentKey, Exchange};
use crate::handlers::Handler;
use crate::status::HandlerStatus;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;

pub const LAMBDA_CONTEXT: AttachmentKey = AttachmentKey(4);
pub(crate) const LOAD_METHOD: &str = "LOAD_METHOD";
const CACHE_CONFIGS: &str = "CACHE_CONFIGS";

#[derive(Debug)]
pub struct HandlerExecutionError {
    message: String,
}

impl Display for HandlerExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fatal error: {}", self.message)
    }
}

impl std::error::Error for HandlerExecutionError {}

//fn load_handlers() -> Vec<Box<dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context, Err = (), HandlerStatus = ()>>>
//{
//    let ev_load_method = match match env::var(LOAD_METHOD) {
//        Ok(v) => LoadMethod::from_str(v.as_str()),
//        Err(_) => Ok(LoadMethod::Default),
//    } {
//        Ok(method) => method,
//        Err(_) => todo!("Handle invalid load method from env variables."),
//    };
//
//    let ev_cache_configs: bool = match env::var(CACHE_CONFIGS) {
//        Ok(v) => v.parse::<bool>().unwrap_or_default(),
//        Err(_) => false,
//    };
//
//    todo!()
//}

const AUDIT_ATTACHMENT: AttachmentKey = AttachmentKey(11);

pub(crate) async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    // TODO - load handler configuration (from cache if possible)...

    let middleware: Vec<
        Box<
            dyn Handler<
                    ApiGatewayProxyRequest,
                    ApiGatewayProxyResponse,
                    Context,
                    Err = HandlerExecutionError,
                    Status = HandlerStatus,
                > + Send,
        >,
    > = vec![];
    let mut executor = LambdaMiddlewareExecutor::new(middleware);
    let (payload, context) = event.into_parts();
    let mut exchange: Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> =
        Exchange::new();
    exchange.save_input(payload);
    exchange.add_metadata(context);

    // TODO - handle auditing at the end of the request...
    exchange
        .attachments_mut()
        .add_attachment::<HashMap<String, String>>(
            AUDIT_ATTACHMENT,
            Box::new(HashMap::<String, String>::new()),
        );

    // TODO - Change the output of each handler from an empty result, to a status object. (exception handler)...
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
            dyn Handler<
                    ApiGatewayProxyRequest,
                    ApiGatewayProxyResponse,
                    Context,
                    Err = HandlerExecutionError,
                    Status = HandlerStatus,
                > + Send,
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
                        Err = HandlerExecutionError,
                        Status = HandlerStatus,
                    > + Send,
            >,
        >,
    ) -> Self {
        Self {
            middlewares: middleware,
        }
    }
}
