use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct EchoRequestHandlerConfig {
    pub enabled: bool,
    pub echo_headers: bool,
    pub static_body: Option<String>
}
