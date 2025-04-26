use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub(crate) struct ModifyHeaderKey(pub String);

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct ModifyHeaderValue(pub String);

#[derive(Deserialize, Serialize, Default, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub(crate) struct PathPrefix(pub String);

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct HeaderHandlerConfig {
    pub enabled: bool,
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
    pub path_prefix_header: HashMap<PathPrefix, PathHeaderHandlerConfig>,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct PathHeaderHandlerConfig {
    pub request: ModifyHeaderHandlerConfig,
    pub response: ModifyHeaderHandlerConfig,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub(crate) struct ModifyHeaderHandlerConfig {
    pub update: HashMap<ModifyHeaderKey, ModifyHeaderValue>,
    pub remove: Vec<ModifyHeaderKey>,
}