use std::fs::File;
use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = "trace.json";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TraceabilityHandlerConfig {
    pub enabled: bool,
    pub autogen_correlation_id: bool,
    pub correlation_header_name: String,
    pub traceability_header_name: String,
    pub correlation_logging_field_name: String,
    pub traceability_logging_field_name: String,
    pub add_trace_to_response: bool,
}

impl TraceabilityHandlerConfig {
    pub fn new(base_config_path: &str) -> Self {
        let file = File::open(format!("{}{}{}", base_config_path, "/", CONFIG_NAME)).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::implementation::traceability::config::TraceabilityHandlerConfig;

    #[test]
    fn test_load_config() {
        let config = TraceabilityHandlerConfig::new("./test_resources/config");
        assert_eq!(config.enabled, true);
        assert_eq!(config.autogen_correlation_id, true);
        assert_eq!(config.correlation_header_name, "x-correlation");
    }
}