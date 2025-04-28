use std::collections::HashMap;
use serde::{Deserialize};

#[derive(Deserialize, Default)]
pub(crate) struct LambdaProxyHandlerConfig {
    pub enabled: bool,
    pub functions: HashMap<String, String>
}

