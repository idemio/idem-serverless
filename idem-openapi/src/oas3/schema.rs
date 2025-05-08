use crate::oas3::spec_extensions;
use std::{collections::HashMap, fmt};
use derive_more::derive::{Display, Error};
use serde::{Deserialize, Deserializer, Serialize};
use crate::oas3::object_ref::{FromRef, Node, Ref, RefError, RefType};
use crate::oas3::Spec;

/// Schema errors.
#[derive(Debug, Clone, PartialEq, Display, Error)]
pub enum Error {
    /// Missing type field.
    #[display("Missing type field")]
    NoType,

    /// Unknown type.
    #[display("Unknown type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    /// Required property list specified for a non-object schema.
    #[display("Required property list specified for a non-object schema")]
    RequiredSpecifiedOnNonObject,
}

/// Single schema type.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    /// Boolean schema type.
    Boolean,

    /// Integer schema type.
    Integer,

    /// Number schema type.
    Number,

    /// String schema type.
    String,

    /// Array schema type.
    Array,

    /// Object schema type.
    Object,

    /// Null schema type.
    Null,
}

/// Set of schema types.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeSet {
    /// Single schema type specified.
    Single(Type),

    /// Multiple possible schema types specified.
    Multiple(Vec<Type>),
}

impl TypeSet {
    /// Returns `true` if this type-set contains the given type.
    pub fn contains(&self, type_: Type) -> bool {
        match self {
            TypeSet::Single(single_type) => *single_type == type_,
            TypeSet::Multiple(type_set) => type_set.contains(&type_),
        }
    }

    /// Returns `true` if this type-set is `object` or `[object, 'null']`.
    pub fn is_object_or_nullable_object(&self) -> bool {
        match self {
            TypeSet::Single(Type::Object) => true,
            TypeSet::Multiple(set) if set == &[Type::Object] => true,
            TypeSet::Multiple(set) if set == &[Type::Object, Type::Null] => true,
            TypeSet::Multiple(set) if set == &[Type::Null, Type::Object] => true,
            _ => false,
        }
    }

    /// Returns `true` if this type-set is `array` or `[array, 'null']`.
    pub fn is_array_or_nullable_array(&self) -> bool {
        match self {
            TypeSet::Single(Type::Array) => true,
            TypeSet::Multiple(set) if set == &[Type::Array] => true,
            TypeSet::Multiple(set) if set == &[Type::Array, Type::Null] => true,
            TypeSet::Multiple(set) if set == &[Type::Null, Type::Array] => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct ObjectSchema {
    #[serde(rename = "allOf", default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<Node<ObjectSchema>>,
    #[serde(rename = "anyOf", default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<Node<ObjectSchema>>,
    #[serde(rename = "oneOf", default, skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<Node<ObjectSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Node<ObjectSchema>>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Node<ObjectSchema>>,
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<Schema>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<TypeSet>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<serde_json::Value>,
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_value: Option<serde_json::Value>,
    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<serde_json::Number>,
    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Number>,
    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<serde_json::Number>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    #[serde(rename = "uniqueItems", skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    #[serde(rename = "maxProperties", skip_serializing_if = "Option::is_none")]
    pub max_contains: Option<u64>,
    #[serde(rename = "minProperties", skip_serializing_if = "Option::is_none")]
    pub min_contains: Option<u64>,
    #[serde(rename = "maxProperties", skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,
    #[serde(rename = "minProperties", skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    // TODO: missing fields
    // - dependentRequired

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(rename = "readOnly", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(rename = "writeOnly", skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,

    #[serde(
        default,
        deserialize_with = "distinguish_missing_and_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub example: Option<serde_json::Value>,

    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl ObjectSchema {
    pub fn is_nullable(&self) -> Option<bool> {
        Some(match self.schema_type.as_ref()? {
            TypeSet::Single(type_) => *type_ == Type::Null,
            TypeSet::Multiple(set) => set.contains(&Type::Null),
        })
    }
}

impl FromRef for ObjectSchema {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        todo!()
//        let refpath = path.parse::<Ref>()?;
//
//        match refpath.kind {
//            RefType::Schema => spec
//                .components
//                .as_ref()
//                .and_then(|cs| cs.schemas.get(&refpath.name))
//                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
//                .and_then(|oor| oor.resolve(spec)),
//
//            typ => Err(RefError::MismatchedType(typ, RefType::Schema)),
//        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    pub property_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BooleanSchema(pub bool);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Schema {
    Boolean(BooleanSchema),
    Object(Box<Node<ObjectSchema>>),
}

/// Considers any value that is present as `Some`, including `null`.
fn distinguish_missing_and_null<'de, T, D>(de: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de> + fmt::Debug,
    D: Deserializer<'de>,
{
    T::deserialize(de).map(Some)
}