use std::collections::HashMap;
use std::fs::File;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "proxy-lambda.json";

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct LambdaProxyHandlerConfig {
    pub enabled: bool,
    pub functions: HashMap<String, String>,
    pub region: String,
    pub endpoint_override: String,
    pub api_call_timeout: u32,
    pub log_type: String,
    pub metrics_injection: bool,
    pub metrics_name: String,
}

impl LambdaProxyHandlerConfig {
    fn load(base_config_path: &str) -> Self {
        let file = File::open(format!("{}{}{}", base_config_path, "/", CONFIG_FILE)).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}

