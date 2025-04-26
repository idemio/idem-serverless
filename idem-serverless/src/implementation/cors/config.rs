use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct CorsHandlerConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub path_prefix_cors_config: HashMap<String, CorsHandlerPathConfig>,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct CorsHandlerPathConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
}