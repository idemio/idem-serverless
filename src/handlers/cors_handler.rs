use crate::exchange::Exchange;
use crate::handlers::Handler;
use aws_sdk_lambda::primitives::event_stream::Header;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::http::HeaderValue;
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

const ACCESS_CONTROL_REQUEST_METHOD_HEADER_KEY: &str = "Access-Control-Request-Method";
const ACCESS_CONTROL_ALLOW_ORIGIN_HEADER_KEY: &str = "Access-Control-Allow-Origin";
const ACCESS_CONTROL_ALLOW_CREDENTIALS_HEADER_KEY: &str = "Access-Control-Allow-Credentials";
const ACCESS_CONTROL_EXPOSE_HEADERS_HEADER_KEY: &str = "Access-Control-Expose-Headers";
const ACCESS_CONTROL_MAX_AGE_HEADER_KEY: &str = "Access-Control-Max-Age";
const ACCESS_CONTROL_ALLOW_METHODS_HEADER_KEY: &str = "Access-Control-Allow-Methods";
const ACCESS_CONTROL_ALLOW_HEADERS_HEADER_KEY: &str = "Access-Control-Allow-Headers";
const ORIGIN_HEADER_KEY: &str = "Origin";
const ACCESS_CONTROL_REQUEST_METHOD: &str = "Access-Control-Request-Method";
const ACCESS_CONTROL_REQUEST_HEADERS: &str = "Access-Control-Request-Headers";
const ACCESS_CONTROL_ALLOW_ORIGIN: &str = "Access-Control-Allow-Origin";
const ACCESS_CONTROL_ALLOW_CREDENTIALS: &str = "Access-Control-Allow-Credentials";
const ACCESS_CONTROL_EXPOSE_HEADERS: &str = "Access-Control-Expose-Headers";
const ACCESS_CONTROL_MAX_AGE: &str = "Access-Control-Max-Age";
const ACCESS_CONTROL_ALLOW_METHODS: &str = "Access-Control-Allow-Methods";
const ACCESS_CONTROL_ALLOW_HEADERS: &str = "Access-Control-Allow-Headers";

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub(crate) struct CorsHandlerConfig {
    enabled: bool,
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    path_prefix_cors_config: HashMap<String, CorsHandlerPathConfig>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub(crate) struct CorsHandlerPathConfig {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct CorsHandler {
    config: CorsHandlerConfig,
}

impl CorsHandler {
    fn remove_default_ports(url: &str) -> &str {
        let scheme_pattern = "://";
        let ipv6_start_pattern = "[";
        let ipv6_end_pattern = "]";
        let port_pattern = ":";

        if let Some(index) = url[scheme_pattern.len()..]
            .find(scheme_pattern)
            .map(|i| i + scheme_pattern.len())
        {
            let scheme = &url[..index];
            let mut after_scheme_index = scheme.len() + scheme_pattern.len();
            if let Some(ipv6_start_index) = url[after_scheme_index..]
                .find(ipv6_start_pattern)
                .map(|i| i + ipv6_start_pattern.len())
            {
                after_scheme_index = url[ipv6_start_index..]
                    .find(ipv6_end_pattern)
                    .map(|i| i + ipv6_end_pattern.len())
                    .unwrap();
            }

            if let Some(port_index) = url[after_scheme_index..]
                .find(port_pattern)
                .map(|i| i + port_pattern.len())
            {
                let port = url[after_scheme_index + port_index..]
                    .parse::<i32>()
                    .unwrap();
                if (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
                    return &url[..after_scheme_index + port_index - 1];
                }
            }
        }
        url
    }
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for CorsHandler {
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
            if self.config.enabled {
                let request = exchange.input().unwrap();
                if let Some(origin_header) = request
                    .headers
                    .iter()
                    .find(|(k, v)| k.to_string().to_lowercase() == ORIGIN_HEADER_KEY.to_lowercase())
                {
                    let origin_header_value =
                        Self::remove_default_ports(origin_header.1.to_str().unwrap());

                    let mut exchange_allowed_origins = self.config.allowed_origins.clone();
                    let mut exchange_allowed_methods = self.config.allowed_methods.clone();

                    /* check path specific configuration */
                    if !self.config.path_prefix_cors_config.is_empty() {
                        let request_path = request.path.clone().unwrap_or("/".to_string());
                        let path_config = self
                            .config
                            .path_prefix_cors_config
                            .iter()
                            .find(|(k, _)| request_path.starts_with(k.as_str()))
                            .map(|(_, v)| v.clone());

                        if path_config.is_some() {
                            let path_config = path_config.unwrap();
                            exchange_allowed_origins.extend(path_config.allowed_origins);
                            exchange_allowed_methods.extend(path_config.allowed_methods);
                        }
                    }

                    /* check if preflight */
                    if request.http_method.eq("OPTIONS") {
                        let mut response = ApiGatewayProxyResponse::default();
                        if exchange_allowed_origins
                            .iter()
                            .any(|origin| origin.to_lowercase().eq(origin_header_value))
                        {
                            response.headers.insert(
                                ACCESS_CONTROL_ALLOW_ORIGIN,
                                HeaderValue::from_str(origin_header_value).unwrap(),
                            );
                            response
                                .headers
                                .insert("Vary", HeaderValue::from_str(ORIGIN_HEADER_KEY).unwrap());
                        } else {
                            /* invalid origin, early return */
                            response.status_code = 403;
                            exchange.save_output(response);
                            return Ok(());
                        }

                        response.headers.insert(
                            ACCESS_CONTROL_ALLOW_METHODS,
                            HeaderValue::from_str(
                                exchange_allowed_methods
                                    .iter()
                                    .map(|x| x.to_string() + ",")
                                    .collect::<Vec<_>>()
                                    .join(",")
                                    .as_str(),
                            )
                            .unwrap(),
                        );

                        if let Some((_, ac_header_value)) =
                            request.headers.iter().find(|(header_key, _)| {
                                header_key.to_string().to_lowercase()
                                    == ACCESS_CONTROL_REQUEST_HEADERS.to_lowercase()
                            })
                        {
                            response
                                .headers
                                .insert(ACCESS_CONTROL_ALLOW_HEADERS, ac_header_value.clone());
                        } else {
                            response.headers.insert(
                                ACCESS_CONTROL_ALLOW_HEADERS,
                                HeaderValue::from_str(
                                    "Content-Type, WWW-Authenticate, Authorization",
                                )
                                .unwrap(),
                            );
                        }

                        response.headers.insert(
                            ACCESS_CONTROL_ALLOW_CREDENTIALS,
                            HeaderValue::from_str("true").unwrap(),
                        );
                        response.headers.insert(
                            ACCESS_CONTROL_MAX_AGE,
                            HeaderValue::from_str("3600").unwrap(),
                        );
                    } else {
                        if !exchange_allowed_origins
                            .iter()
                            .any(|origin| origin.to_lowercase().eq(origin_header_value))
                        {
                            // TODO - Handle validation failure return.
                            return Err(());
                        }
                    }

                    exchange.add_output_listener(|x| {
                        let input = exchange.input().unwrap();
                        if let Some((_, request_origin_header_value)) =
                            input.headers.iter().find(|(k, _)| {
                                k.to_string().to_lowercase() == ORIGIN_HEADER_KEY.to_lowercase()
                            })
                        {
                            let output = exchange.output_mut().unwrap();
                            output.headers.insert(
                                ACCESS_CONTROL_ALLOW_ORIGIN,
                                request_origin_header_value.clone(),
                            );
                        }
                    })
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod test {
    use crate::handlers::cors_handler::CorsHandler;

    #[test]
    fn test_string_find() {
        let http_url = "http://testurl.com:80";
        let sanitized_url = CorsHandler::remove_default_ports(http_url);
        assert_eq!(sanitized_url, "http://testurl.com");

        let http_url = "http://testurl.com:8080";
        let sanitized_url = CorsHandler::remove_default_ports(http_url);
        assert_eq!(sanitized_url, "http://testurl.com:8080");

        let http_url = "http://[2001:db8:4006:812::200e]:80";
        let sanitized_url = CorsHandler::remove_default_ports(http_url);
        assert_eq!(sanitized_url, "http://[2001:db8:4006:812::200e]");
    }
}
