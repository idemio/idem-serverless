use crate::handler::LambdaExchange;
use async_trait::async_trait;
use idem_handler::exchange::AttachmentKey;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub struct ModifyHeaderKey(pub String);

#[derive(Deserialize, Default, Clone)]
pub struct ModifyHeaderValue(pub String);

#[derive(Deserialize, Default, PartialOrd, PartialEq, Hash, Eq)]
pub struct PathPrefix(pub String);

#[derive(Deserialize, Default)]
pub struct HeaderHandlerConfig {
    pub enabled: bool,
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
    pub path_prefix_header: HashMap<PathPrefix, PathHeaderHandlerConfig>,
}

#[derive(Deserialize, Default)]
pub struct PathHeaderHandlerConfig {
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
}

#[derive(Deserialize, Default)]
pub struct ModifyHeaderHandlerConfig {
    pub update: HashMap<ModifyHeaderKey, ModifyHeaderValue>,
    pub remove: Vec<ModifyHeaderKey>,
}

#[derive(ConfigurableHandler)]
pub struct HeaderHandler {
    config: Config<HeaderHandlerConfig>,
}

impl HeaderHandler {
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

const REMOVE_RESPONSE_HEADER_ATTACHMENT_KEY: AttachmentKey =
    AttachmentKey("remove_response_headers");
const UPDATE_RESPONSE_HEADER_ATTACHMENT_KEY: AttachmentKey =
    AttachmentKey("update_response_headers");

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for HeaderHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, HandlerExecutionError> {
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        let request = exchange.input().unwrap();
        let request_path = request.path.as_deref().unwrap_or("/");

        let mut request_remove_headers = vec![];
        let mut request_update_headers = HashMap::new();
        let mut response_remove_headers = vec![];
        let mut response_update_headers = HashMap::new();

        // Gather rules for current path
        request_remove_headers.extend(self.config.get().request.remove.clone());
        request_update_headers.extend(self.config.get().request.update.clone());
        response_remove_headers.extend(self.config.get().response.remove.clone());
        response_update_headers.extend(self.config.get().response.update.clone());

        if let Some((_, path_config)) = self
            .config
            .get()
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

        Ok(HandlerStatus::new(Code::OK))
    }
}
