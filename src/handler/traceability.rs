use async_trait::async_trait;
use idem_handler::exchange::AttachmentKey;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
use lambda_http::{Context, tracing};
use serde::Deserialize;
use crate::handler::LambdaExchange;

#[derive(Deserialize)]
pub struct TraceabilityHandlerConfig {
    pub enabled: bool,
    pub autogen_correlation_id: bool,
    pub correlation_header_name: String,
    pub traceability_header_name: String,
    pub add_trace_to_response: bool,
}

impl Default for TraceabilityHandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            autogen_correlation_id: true,
            traceability_header_name: "x-trace".into(),
            correlation_header_name: "x-correlation".into(),
            add_trace_to_response: true,
        }
    }
}

#[derive(ConfigurableHandler)]
pub struct TraceabilityHandler {
    config: Config<TraceabilityHandlerConfig>,
}

impl TraceabilityHandler {
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

const TRACE_V_ATTACHMENT_KEY: AttachmentKey = AttachmentKey("trace_v");
const CORR_V_ATTACHMENT_KEY: AttachmentKey = AttachmentKey("corr_v");
const CORR_H_ATTACHMENT_KEY: AttachmentKey = AttachmentKey("corr_h");
const TRACE_H_ATTACHMENT_KEY: AttachmentKey = AttachmentKey("trace_h");

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for TraceabilityHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError> {
        tracing::debug!("Traceability handler starts");
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        let request = exchange.input().unwrap();
        let cid_header_name = self.config.get().correlation_header_name.clone();
        let cid = Self::find_or_create_uuid(
            &request.headers,
            &cid_header_name,
            self.config.get().autogen_correlation_id,
        );

        let tid_header_name = self.config.get().traceability_header_name.clone();
        let tid = Self::find_or_create_uuid(&request.headers, &tid_header_name, false);

        if cid.is_some() {
            let cid = cid.unwrap();
            if tid.is_some() {
                let tid = tid.unwrap();
                tracing::info!(
                    "Associate traceability Id {} with correlation Id {}",
                    &tid,
                    &cid
                );

                if self.config.get().add_trace_to_response {
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
            let inserted_header_value: HeaderValue = HeaderValue::from_str(cid.as_str()).unwrap();
            exchange
                .input_mut()
                .unwrap()
                .headers
                .insert(inserted_header_name, inserted_header_value);
        }

        Ok(HandlerStatus::new(Code::OK))
    }
}

#[cfg(test)]
mod test {
    use core::{assert, assert_eq};
    use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
    use crate::handler::traceability::TraceabilityHandler;

    #[test]
    fn test_correlation_id() {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_bytes("x-correlation-id".as_bytes()).unwrap(),
            HeaderValue::from_str("abc123").unwrap(),
        );
        let cid = TraceabilityHandler::find_or_create_uuid(&header_map, "x-correlation-id", true);
        assert!(cid.is_some());
        let cid = cid.unwrap();
        assert_eq!(cid, "abc123".to_string());
    }

    #[test]
    fn test_traceability_header() {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_bytes("x-traceability-id".as_bytes()).unwrap(),
            HeaderValue::from_str("abc123").unwrap(),
        );
        let tid = TraceabilityHandler::find_or_create_uuid(&header_map, "x-traceability-id", false);
        assert!(tid.is_some());
        let tid = tid.unwrap();
        assert_eq!(tid, "abc123".to_string());
    }
}
