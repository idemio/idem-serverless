use crate::oas3::spec_extensions;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::oas3::server::Server;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Link {
    Ref {
        #[serde(rename = "operationRef")]
        operation_ref: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        parameters: HashMap<String, String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,
        #[serde(flatten, with = "spec_extensions")]
        extensions: HashMap<String, Value>,
    },

    Id {
        #[serde(rename = "operationId")]
        operation_id: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        parameters: HashMap<String, String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,
        #[serde(flatten, with = "spec_extensions")]
        extensions: HashMap<String, Value>,
    },
}