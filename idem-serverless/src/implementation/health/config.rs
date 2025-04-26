use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct HealthCheckHandlerConfig {
    pub enabled: bool,
    pub use_json: bool,
    pub timeout: u32,
    pub downstream_enabled: bool,
    pub downstream_function: String,
    pub downstream_function_health_payload: String,
}
