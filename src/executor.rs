use crate::exchange::{AttachmentKey, Exchange, Handler};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use std::collections::HashMap;
use crate::handlers::echo_test_middleware::EchoTestLambdaMiddleware;

pub const LAMBDA_CONTEXT: AttachmentKey = AttachmentKey(4);

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
pub(crate) async fn entry(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {

    let middleware: Vec<Box<dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>> + Send>> = vec![Box::new(EchoTestLambdaMiddleware)];
    let mut executor = LambdaMiddlewareExecutor::new(middleware);
    let (payload, context) = event.into_parts();
    let mut exchange: Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>> = Exchange::new();
    exchange.save_input(payload);
    exchange.add_attachment::<Context>(LAMBDA_CONTEXT, Box::new(context));
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
            dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>>
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
                        HashMap<String, String>,
                    > + Send,
            >,
        >,
    ) -> Self {
        Self {
            middlewares: middleware,
        }
    }
}
