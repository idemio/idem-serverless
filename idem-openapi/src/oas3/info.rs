use crate::oas3::spec_extensions;
use std::collections::HashMap;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// #[serde(rename_all = "lowercase")]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<Url>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Display, Error)]
#[display("Email address is not valid")]
#[non_exhaustive]
pub struct InvalidEmail;
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}