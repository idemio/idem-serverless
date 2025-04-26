use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HealthCheckHandlerConfig {
    pub enabled: bool,
    pub use_json: bool,
    pub timeout: u32,
    pub downstream_enabled: bool,
    pub downstream_function: String,
    pub downstream_function_health_payload: String,
}