use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub struct ModifyHeaderKey(pub String);

#[derive(Deserialize, Default, Clone)]
pub struct ModifyHeaderValue(pub String);

#[derive(Deserialize, Default, PartialOrd, PartialEq, Hash, Eq)]
pub struct PathPrefix(pub String);

#[derive(Deserialize, Default)]
pub struct HeaderHandlerConfig {
    pub enabled: bool,
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
    pub path_prefix_header: HashMap<PathPrefix, PathHeaderHandlerConfig>,
}

#[derive(Deserialize, Default)]
pub struct PathHeaderHandlerConfig {
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
}

#[derive(Deserialize, Default)]
pub struct ModifyHeaderHandlerConfig {
    pub update: HashMap<ModifyHeaderKey, ModifyHeaderValue>,
    pub remove: Vec<ModifyHeaderKey>,
}
