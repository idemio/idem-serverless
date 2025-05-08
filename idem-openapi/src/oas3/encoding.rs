use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::oas3::header::Header;
use crate::oas3::object_ref::Node;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Encoding {
    #[serde(skip_serializing_if = "Option::is_none", rename = "contentType")]
    pub content_type: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, Node<Header>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowReserved")]
    pub allow_reserved: Option<bool>,
}