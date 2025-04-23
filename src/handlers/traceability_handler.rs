use crate::exchange::{AttachmentKey, Exchange};
use crate::handlers::Handler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
use lambda_http::Context;
use log::log;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct TraceabilityHandlerConfig {
    enabled: bool,
    autogen_correlation_id: bool,
    correlation_header_name: String,
    traceability_header_name: String,
    correlation_logging_field_name: String,
    traceability_logging_field_name: String,
    add_trace_to_response: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TraceabilityHandler {
    config: TraceabilityHandlerConfig,
}

impl TraceabilityHandler {

    pub(crate) async fn new(config: TraceabilityHandlerConfig) -> Self {
        Self { config }
    }

    fn find_or_create_uuid(
        headers: &HeaderMap,
        header_name: &str,
        gen_uuid: bool,
    ) -> Option<String> {
        match headers
            .iter()
            .find(|(header_key, _)| header_key.to_string().to_lowercase() == header_name)
        {
            Some((_, header_value)) => match header_value.to_str() {
                Ok(header_string) => Some(header_string.to_string()),
                Err(_) => None,
            },
            None => {
                if gen_uuid {
                    Some(uuid::Uuid::new_v4().to_string())
                } else {
                    None
                }
            }
        }
    }
}

const TRACE_V_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(7);
const CORR_V_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(8);
const CORR_H_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(9);
const TRACE_H_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(10);

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for TraceabilityHandler {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        Box::pin(async move {
            let request = exchange.input().unwrap();
            let cid_header_name = self.config.correlation_header_name.clone();
            let cid = Self::find_or_create_uuid(
                &request.headers,
                &cid_header_name,
                self.config.autogen_correlation_id,
            );

            let tid_header_name = self.config.correlation_header_name.clone();
            let tid = Self::find_or_create_uuid(&request.headers, &tid_header_name, false);

            if cid.is_some() {
                let cid = cid.unwrap();
                if tid.is_some() {
                    let tid = tid.unwrap();
                    log::info!(
                        "Associate traceability Id {} with correlation Id {}",
                        &tid,
                        &cid
                    );

                    if self.config.add_trace_to_response {
                        exchange
                            .attachments_mut()
                            .add_attachment::<String>(TRACE_V_ATTACHMENT_KEY, Box::new(tid));
                        exchange
                            .attachments_mut()
                            .add_attachment::<String>(CORR_V_ATTACHMENT_KEY, Box::new(cid.clone()));
                        exchange.attachments_mut().add_attachment::<String>(
                            CORR_H_ATTACHMENT_KEY,
                            Box::new(cid_header_name.clone()),
                        );
                        exchange.attachments_mut().add_attachment::<String>(
                            TRACE_H_ATTACHMENT_KEY,
                            Box::new(tid_header_name),
                        );
                        exchange.add_output_listener(|response, attachments| {
                            if let (Some(cid_header), Some(cid_value)) = (
                                attachments.attachment::<String>(CORR_H_ATTACHMENT_KEY),
                                attachments.attachment::<String>(CORR_V_ATTACHMENT_KEY),
                            ) {
                                response.headers.insert(
                                    HeaderName::from_bytes(cid_header.as_bytes()).unwrap(),
                                    HeaderValue::from_str(cid_value).unwrap(),
                                );
                            }

                            if let (Some(tid_header), Some(tid_value)) = (
                                attachments.attachment::<String>(TRACE_H_ATTACHMENT_KEY),
                                attachments.attachment::<String>(TRACE_V_ATTACHMENT_KEY),
                            ) {
                                response.headers.insert(
                                    HeaderName::from_bytes(tid_header.as_bytes()).unwrap(),
                                    HeaderValue::from_str(tid_value).unwrap(),
                                );
                            }
                        });
                    }
                }

                let inserted_header_name: HeaderName =
                    HeaderName::from_lowercase(cid_header_name.to_lowercase().as_bytes()).unwrap();
                let inserted_header_value: HeaderValue =
                    HeaderValue::from_str(cid.as_str()).unwrap();
                exchange
                    .input_mut()
                    .unwrap()
                    .headers
                    .insert(inserted_header_name, inserted_header_value);
            }

            Ok(())
        })
    }
}
