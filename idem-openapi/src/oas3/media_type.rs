use std::collections::HashMap;
use log::error;
use serde::{Deserialize, Serialize};
use crate::oas3::encoding::Encoding;
use crate::oas3::error::Error;
use crate::oas3::example::Example;
use crate::oas3::object_ref::Node;
use crate::oas3::schema::ObjectSchema;
use crate::oas3::Spec;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct MediaType {
    /// The schema defining the type used for the request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Node<ObjectSchema>>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub examples: Option<MediaTypeExamples>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub encoding: HashMap<String, Encoding>,
}

impl MediaType {
    /// Resolves and returns the JSON schema definition for this media type.
    pub fn schema(&self, spec: &Spec) -> Result<ObjectSchema, Error> {
        self.schema
            .as_ref()
            .unwrap()
            .resolve(spec)
            .map_err(Error::Ref)
    }
    pub fn examples(&self, spec: &Spec) -> HashMap<String, Example> {
        self.examples
            .as_ref()
            .map(|examples| examples.resolve_all(spec))
            .unwrap_or_default()
    }
}

/// Examples for a media type.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MediaTypeExamples {
    Example {
        /// Example of the media type.
        example: serde_json::Value,
    },

    Examples {
        examples: HashMap<String, Node<Example>>,
    },
}

impl Default for MediaTypeExamples {
    fn default() -> Self {
        MediaTypeExamples::Examples {
            examples: HashMap::new(),
        }
    }
}

impl MediaTypeExamples {
    pub fn is_empty(&self) -> bool {
        match self {
            MediaTypeExamples::Example { .. } => false,
            MediaTypeExamples::Examples { examples } => examples.is_empty(),
        }
    }

    pub fn resolve_all(&self, spec: &Spec) -> HashMap<String, Example> {
        match self {
            Self::Example { example } => {
                let example = Example {
                    description: None,
                    summary: None,
                    value: Some(example.clone()),
                    extensions: HashMap::default(),
                };

                let mut map = HashMap::new();
                map.insert("default".to_owned(), example);

                map
            }

            Self::Examples { examples } => examples
                .iter()
                .filter_map(|(name, oor)| {
                    oor.resolve(spec)
                        .map(|obj| (name.clone(), obj))
                        .map_err(|err| error!("{}", err))
                        .ok()
                })
                .collect(),
        }
    }
}