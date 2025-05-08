use crate::oas3::spec_extensions;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::oas3::example::Example;
use crate::oas3::media_type::MediaType;
use crate::oas3::object_ref::{FromRef, Node, Ref, RefError, RefType};
use crate::oas3::path_item::ParameterStyle;
use crate::oas3::schema::ObjectSchema;
use crate::oas3::Spec;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Node<ObjectSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub examples: HashMap<String, Node<Example>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl FromRef for Header {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::Header => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.headers.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::Example)),
//        }
    }
}