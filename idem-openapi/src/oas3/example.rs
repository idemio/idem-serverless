use crate::oas3::spec_extensions;
use std::collections::{ HashMap};
use serde::{Deserialize, Serialize};
use crate::oas3::object_ref::{FromRef, Ref, RefError, RefType};
use crate::oas3::Spec;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Example {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Example {
    /// Returns JSON-encoded bytes of this example's value.
    pub fn as_bytes(&self) -> Vec<u8> {
        match self.value {
            Some(ref val) => serde_json::to_string(val).unwrap().as_bytes().to_owned(),
            None => vec![],
        }
    }
}

impl FromRef for Example {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::Example => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.examples.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::Example)),
        //}
    }
}