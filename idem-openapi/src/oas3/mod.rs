use crate::oas3::component::Components;
use crate::oas3::error::Error;
use crate::oas3::external_doc::ExternalDoc;
use crate::oas3::info::Info;
use crate::oas3::path_item::{Operation, PathItem, SecurityRequirement};
use crate::oas3::server::Server;
use http::Method;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use serde_json::Value;
use crate::oas3::object_ref::Node;

pub mod component;
pub mod encoding;
pub mod error;
pub mod example;
pub mod external_doc;
pub mod header;
pub mod info;
pub mod link;
pub mod media_type;
pub mod object_ref;
pub mod path_item;
pub mod schema;
pub mod security_schema;
pub mod server;
pub mod spec_extensions;

const OPENAPI_SUPPORTED_VERSION_RANGE: &str = "~3.1";

/// Deserializes an OpenAPI spec (JSON-format) from a string.
pub fn from_json(json: impl AsRef<str>) -> Result<Spec, serde_json::Error> {
    serde_json::from_str(json.as_ref())
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Spec {
    pub openapi: Node<String>,
    pub info: Node<Info>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Node<Server>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Node<HashMap<String, Node<PathItem>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Node<Components>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<Node<SecurityRequirement>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Node<Tag>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub webhooks: HashMap<String, Node<PathItem>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<Node<ExternalDoc>>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, Value>,
}

impl Spec {
//    pub fn validate_version(&self) -> Result<semver::Version, Error> {
//        let spec_version = &self.openapi;
//        let sem_ver = semver::Version::parse(spec_version)?;
//        let required_version = semver::VersionReq::parse(OPENAPI_SUPPORTED_VERSION_RANGE).unwrap();
//
//        if required_version.matches(&sem_ver) {
//            Ok(sem_ver)
//        } else {
//            Err(Error::UnsupportedSpecFileVersion(sem_ver))
//        }
//    }
//    pub fn operation_by_id(&self, operation_id: &str) -> Option<&Operation> {
//        self.operations()
//            .find(|(_, _, op)| {
//                op.operation_id
//                    .as_deref()
//                    .is_some_and(|id| id == operation_id)
//            })
//            .map(|(_, _, op)| op)
//    }
//    pub fn operation(&self, method: &Method, path: &str) -> Option<&Operation> {
//        let resource = self.paths.as_ref()?.get(path)?;
//
//        match *method {
//            Method::GET => resource.get.as_ref(),
//            Method::POST => resource.post.as_ref(),
//            Method::PUT => resource.put.as_ref(),
//            Method::PATCH => resource.patch.as_ref(),
//            Method::DELETE => resource.delete.as_ref(),
//            Method::HEAD => resource.head.as_ref(),
//            Method::OPTIONS => resource.options.as_ref(),
//            Method::TRACE => resource.trace.as_ref(),
//            _ => None,
//        }
//    }
//    pub fn operations(&self) -> impl Iterator<Item = (String, Method, &Operation)> {
//        let paths = &self.paths;
//
//        let ops = paths
//            .iter()
//            .flatten()
//            .flat_map(|(path, item)| {
//                item.methods()
//                    .into_iter()
//                    .map(move |(method, op)| (path.to_owned(), method, op))
//            })
//            .collect::<Vec<_>>();
//
//        ops.into_iter()
//    }
//    pub fn primary_server(&self) -> Option<&Server> {
//        self.servers.first()
//    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}
