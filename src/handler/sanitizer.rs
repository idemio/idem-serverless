use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use async_trait::async_trait;
use http::HeaderMap;
use idemio::config::Config;
use idemio::exchange::Exchange;
use idemio::handler::Handler;
use idemio::status::{ExchangeState, HandlerStatus};
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::HeaderValue;
use serde_json::{Map, Value};
use tiny_clean::{java_script_encoder::{JavaScriptEncoder, JavaScriptEncoderMode}, xml_encoder::{XmlEncoder, XmlEncoderMode}, uri_encoder::{UriEncoder, UriEncoderMode}};
use crate::handler::LambdaExchange;

// TODO - change tiny-clean to allow serialization of mode enums
// TODO - more encoder types (html, css, cdata, etc.)
#[derive(Deserialize, Serialize, Clone)]
pub enum SanitizerMode {
    JavaScript(u64, bool),
    Uri(u64),
    Xml(u64)
}

impl Default for SanitizerMode {
    fn default() -> Self {
        SanitizerMode::JavaScript(4, true)
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub enum SanitizerSettings {

    #[default]
    Disabled,
    Enabled {
        mode: SanitizerMode,
        ignore_list: Option<Vec<String>>,
        encode_list: Option<Vec<String>>
    }
}
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct SanitizerHandlerConfig {
    pub enabled: bool,
    pub body_sanitizer: SanitizerSettings,
    pub header_sanitizer: SanitizerSettings
}



//#[derive(ConfigurableHandler)]
pub struct SanitizerHandler {
    config: Config<SanitizerHandlerConfig>,
}

impl SanitizerHandler {

    fn java_script_encoder_for_mode(mode: u64, ascii_only: bool) -> Result<JavaScriptEncoder, ()> {
        if mode == 1u64 {
            Ok(JavaScriptEncoder::new(JavaScriptEncoderMode::Block, ascii_only))
        } else if mode == 2u64 {
            Ok(JavaScriptEncoder::new(JavaScriptEncoderMode::Attribute, ascii_only))
        } else if mode == 3u64 {
            Ok(JavaScriptEncoder::new(JavaScriptEncoderMode::Html, ascii_only))
        } else if mode == 4u64 {
            Ok(JavaScriptEncoder::new(JavaScriptEncoderMode::Source, ascii_only))
        } else {
            return Err(())
        }
    }

    async fn sanitize_headers(exchange: &mut LambdaExchange, mode: &SanitizerMode, ignore_list: &Option<Vec<String>>, encode_list: &Option<Vec<String>>) -> Result<(), ()> {

        // TODO - add input_mut
        let headers = match exchange.input_mut().await {
            Ok(input) => {
                &mut input.headers
            }
            Err(_) => return Err(())
        };

        match mode {
            SanitizerMode::JavaScript(mode, ascii_only) => {

                let encoder = match Self::java_script_encoder_for_mode(*mode, *ascii_only) {
                    Ok(encoder) => encoder,
                    Err(_) => return Err(())
                };

                for (header_name, header_value) in headers {
                    if ignore_list.as_ref().is_some_and(|list| list.contains(&header_name.to_string())) {
                        continue;
                    } else if encode_list.as_ref().is_some_and(|list| list.contains(&header_name.to_string())) {
                        *header_value = HeaderValue::from_str(&*encoder.encode(header_value.to_str().unwrap())).unwrap();
                    } else if encode_list.as_ref().is_none() {
                        *header_value = HeaderValue::from_str(&*encoder.encode(header_value.to_str().unwrap())).unwrap();
                    }
                }
                Ok(())
            }
            _ => todo!("Implement header sanitizer for modes")
        }
    }

   async fn sanitize_body(exchange: &mut LambdaExchange, mode: &SanitizerMode, ignore_list: &Option<Vec<String>>, encode_list: &Option<Vec<String>>) -> Result<(), ()> {
        let body: Value = match exchange.input().await {
            Ok(input) => {
                match &input.body {
                    None => return Ok(()),
                    Some(body) => {
                        match serde_json::from_str(&body) {
                            Ok(val) => val,
                            Err(_) => return Err(())
                        }
                    }
                }

            }
            Err(_) => return Err(())
        };
        let mut body = match body.as_object() {
            None => return Ok(()),
            Some(body) => body
        };
        let sanitized_body = match mode {
            SanitizerMode::JavaScript(mode, ascii_only) => {
                let encoder = match Self::java_script_encoder_for_mode(*mode, *ascii_only) {
                    Ok(encoder) => encoder,
                    Err(_) => return Err(())
                };

                let mut sanitized_body: Map<String, Value> = Map::new();
                for (key, value) in body {
                    if ignore_list.as_ref().is_some_and(|list| list.contains(&key)) {
                        sanitized_body.insert(key.clone(), value.clone());
                    } else if encode_list.as_ref().is_some_and(|list| list.contains(&key)) {
                        sanitized_body.insert(key.clone(), Self::sanitize_value(value, ignore_list, encode_list, &encoder));
                    } else if encode_list.as_ref().is_none() {
                        sanitized_body.insert(key.clone(), Self::sanitize_value(value, ignore_list, encode_list, &encoder));
                    }
                }
                sanitized_body
            }
            SanitizerMode::Uri(mode) => {
                todo!("Implement URI encoder for body")
            }
            SanitizerMode::Xml(mode) => {
                todo!("Implement XML encoder for body")
            }
        };
        if let Ok(input) = exchange.input_mut().await {
            if let Ok(value) = serde_json::to_string(&Value::Object(sanitized_body)) {
                input.body = Some(value);
                return Ok(())
            }
        }
        Err(())
    }

    fn sanitize_value(current_value: &Value, ignore_list: &Option<Vec<String>>, encode_list: &Option<Vec<String>>, encoder: &JavaScriptEncoder) -> Value {
        if let Some(value) = current_value.as_object() {
            let mut map_value: Map<String, Value> = Map::new();
            for (key, value) in value {
                if ignore_list.as_ref().is_some_and(|list| list.contains(&key)) {
                    map_value.insert(key.clone(), value.clone());
                } else if encode_list.as_ref().is_some_and(|list| list.contains(&key)) {
                    map_value.insert(key.clone(), Self::sanitize_value(value, ignore_list, encode_list, &encoder));
                } else if encode_list.as_ref().is_none() {
                    map_value.insert(key.clone(), Self::sanitize_value(value, ignore_list, encode_list, &encoder));
                }
            }
            Value::Object(map_value)

        } else if let Some(value) = current_value.as_array() {
            let capacity = value.len();
            let mut array_value: Vec<Value> = Vec::with_capacity(capacity);
            for item in value {
                array_value.push(Self::sanitize_value(item, ignore_list, encode_list, &encoder));
            }
            Value::Array(array_value)
        } else if let Some(value) = current_value.as_str() {
            let string_value = encoder.encode(value);
            Value::String(string_value)
        } else {
            current_value.clone()
        }
    }


}

#[async_trait]
impl Handler<Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>> for SanitizerHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, Infallible> {
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(ExchangeState::DISABLED));
        }
        match &self.config.get().body_sanitizer {
            SanitizerSettings::Disabled => {
                // body disabled, do nothing...
            }
            SanitizerSettings::Enabled {
                mode,
                ignore_list,
                encode_list
            } => {
                if let Err(_) = Self::sanitize_body(exchange, mode, ignore_list, encode_list).await {
                    return Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR));
                }
            }
        }

        match &self.config.get().header_sanitizer {
            SanitizerSettings::Disabled => {
                // header disabled, do nothing...
            }
            SanitizerSettings::Enabled {
                mode,
                ignore_list,
                encode_list
            } => {
                if let Err(_) = Self::sanitize_headers(exchange, mode, ignore_list, encode_list).await {
                    return Ok(HandlerStatus::new(ExchangeState::SERVER_ERROR));
                }
            }
        }
        Ok(HandlerStatus::new(ExchangeState::OK))
    }

    fn name(&self) -> &str {
        "SanitizerHandler"
    }
}
