use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct JwtValidationHandlerConfig {
    enabled: bool,
    jwk_server_url: String,
    jwk_server_path: String,
    audience: String,
}