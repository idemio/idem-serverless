use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ExecutionFlowConfig {
    pub handlers: Vec<String>,
    pub chains: HashMap<String, Vec<String>>,

    /// Path keys can be based on different data types. i.e. URL paths, IP, mac addr, RPC paths, etc.
    /// They hold what handlers should be executed in the 'PrefixConfig' associated.
    pub paths: HashMap<String, PrefixConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PrefixConfig {
    pub method: String,
    pub exec: Vec<String>
}

//impl ExecutionFlowConfig {
//    pub fn handlers(&self) -> &Vec<String> {
//        &self.handlers
//    }
//
//    pub fn chains(&self) -> &HashMap<String, Vec<String>> {
//        &self.chains
//    }
//
//    pub fn paths(&self) -> &HashMap<String, PrefixConfig> {
//        &self.paths
//    }
//}


#[cfg(test)]
mod test {
    use crate::execution_flow_config::ExecutionFlowConfig;

    #[test]
    fn test_read_execution_path_config() {
        let test_config_string = r#"
        {
            "handlers": [
                "ProxyHandler",
                "TraceabilityHandler",
                "HeaderHandler",
                "JwtValidationHandler",
                "MyCustomHandler",
                "HealthCheckHandler"
            ],
            "chains": {
                "default": [
                    "TraceabilityHandler",
                    "JwtValidationHandler",
                    "HeaderHandler"
                ]
            },
            "paths": {
                "/some/resource/path": {
                    "method": "GET",
                    "exec": [
                        "default",
                        "MyCustomHandler",
                        "ProxyHandler"
                    ]
                },
                "/health": {
                    "method": "GET",
                    "exec": [
                        "default",
                        "HealthCheckHandler"
                    ]
                }
            }
        }"#;
        let my_config = serde_json::from_str::<ExecutionFlowConfig>(test_config_string).unwrap();
        assert_eq!(my_config.handlers.len(), 6);
        assert_eq!(my_config.chains.len(), 1);
        assert_eq!(my_config.paths.len(), 2);
    }
}