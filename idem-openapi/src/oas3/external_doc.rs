use crate::oas3::spec_extensions;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

//pub enum ObjectOrReference<T> {
//    Ref {
//        #[serde(rename = "$ref")]
//        ref_path: String,
//    },
//    Object(T),
//}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExternalDoc {
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}