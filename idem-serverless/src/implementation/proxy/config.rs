use std::collections::HashMap;
use serde::{Deserialize};

const CONFIG_FILE: &str = "proxy-lambda.json";

#[derive(Deserialize, Default)]
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

