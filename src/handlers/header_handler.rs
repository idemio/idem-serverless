use crate::exchange::{AttachmentKey, Exchange};
use crate::handlers::Handler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Deserialize, Serialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub(crate) struct ModifyHeaderKey(pub String);

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct ModifyHeaderValue(pub String);

#[derive(Deserialize, Serialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub(crate) struct PathPrefix(pub String);

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct HeaderHandlerConfig {
    enabled: bool,
    request: ModifyHeaderHandlerConfig,
    response: ModifyHeaderHandlerConfig,
    path_prefix_header: HashMap<PathPrefix, PathHeaderHandlerConfig>,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct PathHeaderHandlerConfig {
    request: ModifyHeaderHandlerConfig,
    response: ModifyHeaderHandlerConfig,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct ModifyHeaderHandlerConfig {
    update: HashMap<ModifyHeaderKey, ModifyHeaderValue>,
    remove: Vec<ModifyHeaderKey>,
}

#[derive(Default, Clone)]
pub(crate) struct HeaderHandler {
    config: HeaderHandlerConfig,
}

impl HeaderHandler {

    pub(crate) async fn new(config: HeaderHandlerConfig) -> Self {
        Self {
            config,
        }
    }

    fn remove_headers(headers: &mut HeaderMap, remove_headers: Vec<ModifyHeaderKey>) {
        for header in remove_headers {
            headers.remove(header.0).unwrap();
        }
    }

    fn update_headers(
        headers: &mut HeaderMap,
        update_headers: HashMap<ModifyHeaderKey, ModifyHeaderValue>,
    ) {
        for (header_key, header_value) in update_headers {
            headers.insert(
                HeaderName::from_bytes(header_key.0.as_bytes()).unwrap(),
                HeaderValue::from_str(header_value.0.as_str()).unwrap(),
            );
        }
    }
}

const REMOVE_RESPONSE_HEADER_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(5);
const UPDATE_RESPONSE_HEADER_ATTACHMENT_KEY: AttachmentKey = AttachmentKey(6);

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for HeaderHandler {
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
            if !self.config.enabled {
                return Ok(());
            }

            let request = exchange.input().unwrap();
            let request_path = request.path.as_deref().unwrap_or("/");

            let mut request_remove_headers = vec![];
            let mut request_update_headers = HashMap::new();
            let mut response_remove_headers = vec![];
            let mut response_update_headers = HashMap::new();

            // Gather rules for current path
            request_remove_headers.extend(self.config.request.remove.clone());
            request_update_headers.extend(self.config.request.update.clone());
            response_remove_headers.extend(self.config.response.remove.clone());
            response_update_headers.extend(self.config.response.update.clone());

            if let Some((_, path_config)) = self
                .config
                .path_prefix_header
                .iter()
                .find(|(path_prefix, _)| request_path.starts_with(&path_prefix.0))
            {
                request_remove_headers.extend(path_config.request.remove.clone());
                request_update_headers.extend(path_config.request.update.clone());
                response_remove_headers.extend(path_config.response.remove.clone());
                response_update_headers.extend(path_config.response.update.clone());
            }

            /* handle header request changes */
            Self::update_headers(
                &mut exchange.input_mut().unwrap().headers,
                request_update_headers,
            );

            Self::remove_headers(
                &mut exchange.input_mut().unwrap().headers,
                request_remove_headers,
            );

            /* handle header response changes */
            exchange
                .attachments_mut()
                .add_attachment::<Vec<ModifyHeaderKey>>(
                    REMOVE_RESPONSE_HEADER_ATTACHMENT_KEY,
                    Box::new(response_remove_headers),
                );
            exchange
                .attachments_mut()
                .add_attachment::<HashMap<ModifyHeaderKey, ModifyHeaderValue>>(
                    UPDATE_RESPONSE_HEADER_ATTACHMENT_KEY,
                    Box::new(response_update_headers),
                );

            exchange.add_output_listener(|response, attachments| {
                if let Some(remove_headers) = attachments
                    .attachment::<Vec<ModifyHeaderKey>>(REMOVE_RESPONSE_HEADER_ATTACHMENT_KEY)
                {
                    Self::remove_headers(&mut response.headers, remove_headers.clone())
                }

                if let Some(update_headers) = attachments
                    .attachment::<HashMap<ModifyHeaderKey, ModifyHeaderValue>>(
                        UPDATE_RESPONSE_HEADER_ATTACHMENT_KEY,
                    )
                {
                    Self::update_headers(&mut response.headers, update_headers.clone())
                }
            });

            Ok(())
        })
    }
}
