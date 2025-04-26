use crate::implementation::proxy::config::LambdaProxyHandlerConfig;
use crate::implementation::proxy::handler::LambdaProxyHandler;
use crate::implementation::traceability::config::TraceabilityHandlerConfig;
use crate::implementation::traceability::handler::TraceabilityHandler;
use crate::implementation::{LambdaHandlerExecutor, LambdaHandlers};
use idem_handler::exchange::{AttachmentKey, Exchange};
use idem_handler::handler::Handler;
use idem_handler::status::Code;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::{Context, Error, LambdaEvent};
use std::collections::HashMap;
use std::fmt::Display;

pub const LAMBDA_CONTEXT: AttachmentKey = AttachmentKey(4);

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
pub type LambdaHandler =
    Box<dyn Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> + Send>;

//const AUDIT_ATTACHMENT: AttachmentKey = AttachmentKey(11);

pub(crate) async fn entry(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let mut handlers: Vec<LambdaHandlers> = vec![];

    handlers.push(LambdaHandlers::TraceabilityHandler(
        TraceabilityHandler::new(TraceabilityHandlerConfig {
            enabled: true,
            autogen_correlation_id: true,
            traceability_header_name: "x-traceability-id".into(),
            correlation_header_name: "x-correlation-id".into(),
            add_trace_to_response: true,
            ..Default::default()
        })
        .await,
    ));

    handlers.push(LambdaHandlers::ProxyHandler(LambdaProxyHandler::new(LambdaProxyHandlerConfig {
        enabled: true,
        functions: HashMap::from([("/path/to/resource@POST".to_string(), "arn:aws:lambda:ca-central-1:173982495217:function:test-lambda-function-destination".to_string())]),
        ..Default::default()
    }).await));

    let executor = LambdaHandlerExecutor::new(handlers);
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

    'handler_exec: for handler in &executor.handlers {
        match handler.process(&mut exchange).await {
            Ok(status) => {
                if status
                    .code()
                    .any_flags(Code::TIMEOUT | Code::SERVER_ERROR | Code::CLIENT_ERROR)
                {
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
