use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, ServerVariable>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ServerVariable {
    pub default: String,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub substitutions_enum: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}