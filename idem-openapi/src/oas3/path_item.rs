use crate::oas3::{spec_extensions};
use std::collections::{HashMap, HashSet};

use http::Method;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::oas3::error::Error;
use crate::oas3::example::Example;
use crate::oas3::external_doc::ExternalDoc;
use crate::oas3::header::Header;
use crate::oas3::link::Link;
use crate::oas3::media_type::MediaType;
use crate::oas3::object_ref::{FromRef, Node, Ref, RefError, RefType};
use crate::oas3::schema::ObjectSchema;
use crate::oas3::security_schema::Callback;
use crate::oas3::server::Server;
use crate::oas3::Spec;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityRequirement(pub HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParameterIn {
    Path,
    Query,
    Header,
    Cookie,
}

/// Parameter style.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParameterStyle {
    Matrix,
    Label,
    Form,
    Simple,
    SpaceDelimited,
    PipeDelimited,
    DeepObject,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterIn,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_empty_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_reserved: Option<bool>,
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

impl FromRef for Parameter {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>
    where
        Self: Sized,
    {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::Parameter => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.parameters.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::Parameter)),
//        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PathItem {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    PathItem(PathItemSchema),
}



#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PathItemSchema {
    #[serde(skip_serializing_if = "Option::is_none", rename = "$ref")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Node<Parameter>>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Operation {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDoc>,
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Node<Parameter>>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<Node<RequestBody>>,
    pub responses: Option<HashMap<String, Node<Response>>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub callbacks: HashMap<String, Callback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<SecurityRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Operation {
    pub fn request_body(&self, spec: &Spec) -> Result<RequestBody, Error> {
        self.request_body
            .as_ref()
            .unwrap()
            .resolve(spec)
            .map_err(Error::Ref)
    }
    pub fn responses(&self, spec: &Spec) -> HashMap<String, Response> {
        self.responses
            .iter()
            .flatten()
            .filter_map(|(name, oor)| {
                oor.resolve(spec)
                    .map(|obj| (name.clone(), obj))
                    // TODO: find better error solution
                    .map_err(|err| error!("{}", err))
                    .ok()
            })
            .collect()
    }
    pub fn parameters(&self, spec: &Spec) -> Result<Vec<Parameter>, Error> {
        let params = self
            .parameters
            .iter()
            // TODO: find better error solution, maybe vec<result<_>>
            .filter_map(|oor| oor.resolve(spec).map_err(|err| error!("{}", err)).ok())
            .collect();

        Ok(params)
    }
    pub fn parameter(&self, search: &str, spec: &Spec) -> Result<Option<Parameter>, Error> {
        let param = self
            .parameters(spec)?
            .iter()
            .find(|param| param.name == search)
            .cloned();
        Ok(param)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct RequestBody {

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub content: HashMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

impl FromRef for RequestBody {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>
    where
        Self: Sized,
    {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::RequestBody => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.request_bodies.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::RequestBody)),
//        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Response {

    pub description: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, Node<Header>>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub content: HashMap<String, MediaType>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub links: HashMap<String, Node<Link>>,

    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, Value>,
}

impl FromRef for Response {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::Response => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.responses.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::Response)),
//        }
    }
}

//impl PathItem {
//    /// Returns iterator over this path's provided operations, keyed by method.
//    pub fn methods(&self) -> impl IntoIterator<Item = (Method, &Operation)> {
//        let mut methods = vec![];
//
//        macro_rules! push_method {
//            ($field:ident, $method:ident) => {{
//                if let Some(ref op) = self.$field {
//                    methods.push((Method::$method, op))
//                }
//            }};
//        }
//
//        push_method!(get, GET);
//        push_method!(put, PUT);
//        push_method!(post, POST);
//        push_method!(delete, DELETE);
//        push_method!(options, OPTIONS);
//        push_method!(head, HEAD);
//        push_method!(patch, PATCH);
//        push_method!(trace, TRACE);
//        push_method!(trace, TRACE);
//
//        methods
//    }
//}