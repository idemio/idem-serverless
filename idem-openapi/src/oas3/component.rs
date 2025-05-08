use crate::oas3::{spec_extensions};
use std::collections::{HashMap};

use serde::{Deserialize, Serialize};
use crate::oas3::example::Example;
use crate::oas3::header::Header;
use crate::oas3::link::Link;
use crate::oas3::object_ref::Node;
use crate::oas3::path_item::{Parameter, PathItem, RequestBody, Response};
use crate::oas3::schema::ObjectSchema;
use crate::oas3::security_schema::{Callback, SecurityScheme};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Components {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, Node<ObjectSchema>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub responses: HashMap<String, Node<Response>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, Node<Parameter>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub examples: HashMap<String, Node<Example>>,
    #[serde(
        rename = "requestBodies",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub request_bodies: HashMap<String, Node<RequestBody>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, Node<Header>>,
    #[serde(
        rename = "pathItems",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub path_items: HashMap<String, Node<PathItem>>,
    #[serde(
        rename = "securitySchemes",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub security_schemes: HashMap<String, Node<SecurityScheme>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub links: HashMap<String, Node<Link>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub callbacks: HashMap<String, Node<Callback>>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}
