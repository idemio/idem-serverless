use crate::exchange::Exchange;
use crate::handlers::Handler;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use std::future::Future;
use std::pin::Pin;
use lambda_http::http::{HeaderMap, HeaderName, HeaderValue};
use log::log;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct TraceabilityHandlerConfig {
    enabled: bool,
    autogen_correlation_id: bool,
    correlation_header_name: String,
    traceability_header_name: String,
    correlation_logging_field_name: String,
    traceability_logging_field_name: String
}

#[derive(Clone, Default)]
pub(crate) struct TraceabilityHandler {
    config: TraceabilityHandlerConfig
}

impl TraceabilityHandler {
    fn find_or_create_uuid(headers: &HeaderMap, header_name: &str, gen_uuid: bool) -> Option<String> {
        match headers.iter().find(|(header_key,_)| header_key.to_string().to_lowercase() == header_name) {
            Some((_,header_value)) => {
                match header_value.to_str() {
                    Ok(header_string) => Some(header_string.to_string()),
                    Err(_) => None
                }
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
            let cid = Self::find_or_create_uuid(&request.headers, &cid_header_name, self.config.autogen_correlation_id);

            let tid_header_name = self.config.correlation_header_name.clone();
            let tid = Self::find_or_create_uuid(&request.headers, &tid_header_name, false);

            if cid.is_some() {

                if tid.is_some() {
                    log::info!("Associate traceability Id {} with correlation Id {}", tid.unwrap(), cid.clone().unwrap());
                }

                let inserted_header_name: HeaderName = HeaderName::from_lowercase(cid_header_name.to_lowercase().as_bytes()).unwrap();
                let inserted_header_value: HeaderValue = HeaderValue::from_str(cid.unwrap().as_str()).unwrap();
                exchange.input_mut().unwrap().headers.insert(inserted_header_name, inserted_header_value);
            }

            Ok(())
        })
    }
}
