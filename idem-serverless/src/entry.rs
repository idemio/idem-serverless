use std::fmt::{Display};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use idem_handler::exchange::{AttachmentKey, Exchange};
use idem_handler::handler::{Handler, HandlerLoader};
use idem_handler::status::Code;
use crate::implementation::HandlerRegister;

pub const LAMBDA_CONTEXT: AttachmentKey = AttachmentKey(4);

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
pub type LambdaHandler = Box<
    dyn Handler<
        ApiGatewayProxyRequest,
        ApiGatewayProxyResponse,
        Context,
    > + Send,
>;

//const AUDIT_ATTACHMENT: AttachmentKey = AttachmentKey(11);

pub(crate) async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {

    let mut middlewares: Vec<LambdaHandler> = vec![];
    if let Ok(trace_handler) = HandlerRegister::async_from_str("idem.TraceabilityHandler").await {
        middlewares.push(trace_handler);
    }

    if let Ok(header_handler) = HandlerRegister::async_from_str("idem.HeaderHandler").await {
        middlewares.push(header_handler);
    }

    if let Ok(proxy_handler) = HandlerRegister::async_from_str("idem.ProxyHandler").await {
        middlewares.push(proxy_handler);
    }

    let mut executor = LambdaMiddlewareExecutor::new(middlewares);
    let (payload, context) = event.into_parts();
    let mut exchange: LambdaExchange = Exchange::new();
    exchange.save_input(payload);
    exchange.add_metadata(context);

    // TODO - handle auditing at the end of the request...
    //    exchange
    //        .attachments_mut()
    //        .add_attachment::<HashMap<String, String>>(
    //            AUDIT_ATTACHMENT,
    //            Box::new(HashMap::<String, String>::new()),
    //        );

    'handler_exec: for middleware in &executor.middlewares {
        match middleware.process(&mut exchange).await {
            Ok(status) => {

                if status.code().any_flags(Code::TIMEOUT | Code::SERVER_ERROR | Code::CLIENT_ERROR) {
                    todo!("Handle exception here")
                } else if status.code().any_flags(Code::CONTINUE) {
                    todo!("Handle continue flow here")
                } else if status.code().any_flags(Code::OK | Code::DISABLED) {
                    continue;
                } else if status.code().all_flags(Code::REQUEST_COMPLETED) {
                    break 'handler_exec;
                }
            }
            Err(err) => {
                todo!("Return with exception handler")
            }
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
                > + Send,
            >,
        >,
    ) -> Self {
        Self {
            middlewares: middleware,
        }
    }
}