pub mod node_finder;
use crate::node_finder::{JsonPath, OpenApiNodeFinder};
use dashmap::DashMap;
use jsonschema::{Resource, Validator};
use oas3::{Spec};
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::sync::Arc;
use oas3::spec::{ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, SchemaType};

pub(crate) fn validate_with_schema(value: &Value, schema: &ObjectSchema) -> Result<(), ()> {
    if let Ok(schema_as_value) = serde_json::to_value(schema) {
        if let Ok(_) = jsonschema::validate(&schema_as_value, value) {
            return Ok(());
        }
    }
    Err(())
}

pub(crate) fn try_cast_to_type(
    target_segment: &str,
    schema_type: &SchemaType,
) -> Result<Value, ()> {
    match schema_type {
        SchemaType::Boolean => {
            let cast: bool = target_segment.parse().unwrap();
            Ok(json!(cast))
        }
        SchemaType::Integer => {
            let cast: i64 = target_segment.parse().unwrap();
            Ok(json!(cast))
        }
        SchemaType::Number => {
            let cast: f64 = target_segment.parse().unwrap();
            Ok(json!(cast))
        }
        SchemaType::String => Ok(json!(target_segment)),

        // invalid type for path parameter
        _ => Err(()),
    }
}

#[derive(Debug)]
pub enum OpenApiValidationError {
    InvalidSchema(String),
    InvalidRequest(String),
    InvalidResponse(String),
    InvalidPath(String),
    InvalidMethod(String),
    InvalidContentType(String),
    InvalidAccept(String),
    InvalidQueryParameters(String),
    InvalidHeaders(String),
}

impl Display for OpenApiValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiValidationError::InvalidSchema(msg) => write!(f, "InvalidSchema: {}", msg),
            OpenApiValidationError::InvalidRequest(msg) => write!(f, "InvalidRequest: {}", msg),
            OpenApiValidationError::InvalidResponse(msg) => write!(f, "InvalidResponse: {}", msg),
            OpenApiValidationError::InvalidPath(msg) => write!(f, "InvalidPath: {}", msg),
            OpenApiValidationError::InvalidMethod(msg) => write!(f, "InvalidMethod: {}", msg),
            OpenApiValidationError::InvalidContentType(msg) => {
                write!(f, "InvalidContentType: {}", msg)
            }
            OpenApiValidationError::InvalidAccept(msg) => write!(f, "InvalidAccept: {}", msg),
            OpenApiValidationError::InvalidQueryParameters(msg) => {
                write!(f, "InvalidQueryParameters: {}", msg)
            }
            OpenApiValidationError::InvalidHeaders(msg) => write!(f, "InvalidHeaders: {}", msg),
        }
    }
}

impl std::error::Error for OpenApiValidationError {}

pub struct OpenApiValidator {
    specification: Spec,
    root_schema: Value,
}

impl OpenApiValidator {
    const PATH_SPLIT: char = '/';
    const PATH_PARAM_LEFT: char = '{';
    const PATH_PARAM_RIGHT: char = '}';
    const ROOT_SCHEMA_ID: &'static str = "@@root";
    const PATHS_KEY: &'static str = "paths";
    const OPERATIONS_KEY: &'static str = "operations";
    const PARAMETERS_KEY: &'static str = "parameters";
    const REQUEST_BODY_KEY: &'static str = "requestBody";
    const CONTENT_KEY: &'static str = "content";
    const SCHEMA_KEY: &'static str = "schema";
    const SCHEMAS_KEY: &'static str = "schemas";
    const PARAMETER_IN_KEY: &'static str = "in";
    const PARAMETER_NAME_KEY: &'static str = "name";
    const PARAMETER_REQUIRED_KEY: &'static str = "required";

    pub fn from_file(specification_filename: &str) -> Self {
        let file = fs::read_to_string(specification_filename).unwrap();
        Self::from_json_string(file)
    }

    pub fn from_json_string(json_contents: String) -> Self {
        let mut spec: Value = serde_json::from_str(&json_contents).unwrap();
        spec["$id"] = json!(Self::ROOT_SCHEMA_ID);
        let traversable_spec = oas3::from_json(json_contents).unwrap();
        Self {
            specification: traversable_spec,
            root_schema: spec,
        }
    }

    fn object_schema_to_value(schema: &ObjectSchema) -> Result<Value, OpenApiValidationError> {
        match serde_json::to_value(schema) {
            Ok(val) => Ok(val),
            Err(e) => Err(OpenApiValidationError::InvalidSchema(format!(
                "Failed to convert schema to value: {}",
                e.to_string()
            ))),
        }
    }

    fn validate_with_schema(
        value: &Value,
        schema: &ObjectSchema,
    ) -> Result<(), OpenApiValidationError> {
        let schema_as_value = match Self::object_schema_to_value(schema) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        match jsonschema::validate(&schema_as_value, value) {
            Ok(_) => Ok(()),
            Err(e) => Err(OpenApiValidationError::InvalidSchema(format!(
                "Invalid schema: {}",
                e.to_string()
            ))),
        }
    }

    fn validate_schema_from_pointer(
        &self,
        instance: &Value,
        json_path: &JsonPath,
    ) -> Result<(), OpenApiValidationError> {
        let validator = match get_validator(json_path, &self.root_schema) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match validator.validate(instance) {
            Ok(_) => Ok(()),
            Err(_) => Err(OpenApiValidationError::InvalidSchema(
                "Validation failed".to_string(),
            )),
        }
    }

    fn resolve_request_body(
        &self,
        operation: &Operation,
        content_type: &str,
    ) -> Option<(ObjectSchema, JsonPath)> {
        let request_body_ref = operation
            .request_body
            .as_ref()?
            .resolve(&self.specification)
            .ok()?;
        let mut json_path = JsonPath::new();
        json_path.add_segment(Self::REQUEST_BODY_KEY.to_string());

        let content = request_body_ref.content.get(content_type)?;
        json_path.add_segment(Self::CONTENT_KEY.to_string());
        json_path.add_segment(content_type.to_string());

        let schema = content.schema.as_ref()?.resolve(&self.specification).ok()?;
        json_path.add_segment(Self::SCHEMA_KEY.to_string());

        Some((schema, json_path))
    }

    fn validate_request_headers(
        &self,
        operation: &Operation,
        headers: &HashMap<String, String>,
        _json_path: &JsonPath,
    ) -> Result<(), OpenApiValidationError>
    {
        match Self::filter_and_validate_params(
            headers,
            ParameterIn::Header,
            &operation.parameters,
            &self.specification
        ) {
            true => Ok(()),
            false => Err(OpenApiValidationError::InvalidQueryParameters("Validation failed".to_string())),
        }
    }

    fn validate_request_query_params(
        &self,
        operation: &Operation,
        query_params: &HashMap<String, String>,
        _json_path: &JsonPath,
    ) -> Result<(), OpenApiValidationError>
    {
        match Self::filter_and_validate_params(
            query_params,
            ParameterIn::Query,
            &operation.parameters,
            &self.specification
        ) {
            true => Ok(()),
            false => Err(OpenApiValidationError::InvalidHeaders("Validation failed".to_string())),
        }
    }

    fn validate_request_body(
        &self,
        operation: &Operation,
        body: &Value,
        headers: &HashMap<String, String>,
        path: &JsonPath,
    ) -> Result<(), OpenApiValidationError>
    {
        let mut request_body_path = path.clone();
        let content_type_header = match headers
            .into_iter()
            .find(|(header_name, _)| header_name.to_lowercase().starts_with("content-type"))
        {
            None => {
                return Err(OpenApiValidationError::InvalidRequest(
                    "No content type provided".to_string(),
                ));
            }
            Some((_, header_value)) => header_value,
        };

        let binding = content_type_header.split(";").collect::<Vec<&str>>();
        let content_type_header = match binding.iter().find(|header_value| {
            header_value.starts_with("text")
                || header_value.starts_with("application")
                || header_value.starts_with("multipart")
        }) {
            None => {
                return Err(OpenApiValidationError::InvalidContentType(format!(
                    "Invalid content type provided: {}",
                    content_type_header
                )));
            }
            Some(header_value) => header_value,
        };

        let (_, path) = self
            .resolve_request_body(operation, content_type_header)
            .ok_or_else(|| "Failed to resolve the request body schema".to_string())
            .unwrap();

        request_body_path.append_path(path);
        self.validate_schema_from_pointer(body, &request_body_path)
    }

    pub fn validate_request(
        &self,
        path: &str,
        method: &str,
        body: Option<&Value>,
        headers: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
    ) -> Result<(), OpenApiValidationError>
    {
        match OpenApiNodeFinder::find_matching_operation(path, method, &self.specification, true) {
            Some((operation, path)) => {
                // if body was provided, so we validate it.
                // If a body was provided, the headers must also be provided because we need to find out the content-type
                // This is used to find out which schema to validate against in the openapi specification.
                let body_result = match (body, headers) {
                    (Some(body), Some(headers)) => {
                        self.validate_request_body(&operation, body, headers, &path)
                    }
                    (Some(_), None) => {
                        return Err(OpenApiValidationError::InvalidRequest(
                            "No content type provided".to_string(),
                        ));
                    }
                    (_, _) => Ok(()),
                };

                if let Err(e) = body_result {
                    return Err(e);
                }

                if let Some(headers) = headers {
                    if let Err(e) = self.validate_request_headers(&operation, headers, &path) {
                        return Err(e);
                    }
                }

                if let Some(query_params) = query_params {
                    if let Err(e) = self.validate_request_query_params(&operation, query_params, &path) {
                        return Err(e);
                    }
                }

                Ok(())
            }

            None => Err(OpenApiValidationError::InvalidPath(format!(
                "Could not find matching operation for provided path: {}",
                path
            ))),
        }
    }

    fn filter_and_validate_params(
        given_parameters: &HashMap<String, String>,
        given_parameter_type: ParameterIn,
        operation_parameters: &Vec<ObjectOrReference<Parameter>>,
        spec: &Spec,
    ) -> bool {
        // Filters the current operation parameters to the ones that have the matching
        // 'ParameterIn' type. i.e. ParameterIn::Header, ParameterIn::Query, etc.
        let relevant_parameters = operation_parameters
            .iter()
            .filter(|param| {
                param
                    .resolve(&spec)
                    .is_ok_and(|param| param.location == given_parameter_type)
            })
            .collect::<Vec<&ObjectOrReference<Parameter>>>();

        Self::validate_operation_parameters(given_parameters, &relevant_parameters, &spec)
    }

    fn validate_operation_parameters(
        given_parameters: &HashMap<String, String>,
        operation_parameters_sub_set: &Vec<&ObjectOrReference<Parameter>>,
        specification: &Spec,
    ) -> bool {
        for parameter in operation_parameters_sub_set {
            if let Ok(resolved_param) = parameter.resolve(&specification) {
                if let Some((_, param_value)) = given_parameters
                    .iter()
                    .find(|(param_key, _)| param_key.as_str() == resolved_param.name)
                {
                    if let Some(_) = resolved_param.content {
                    } else if let Some(schema) = resolved_param.schema {
                        if let Ok(schema) = schema.resolve(&specification) {
                            if let Err(_) = Self::validate_with_schema(
                                &Value::String(param_value.clone()),
                                &schema,
                            ) {
                                return false;
                            }
                        }
                    }

                /* if the header is not found, check to see if it's required or not. */
                } else if resolved_param.required.unwrap_or(false) {
                    return false;
                }
            }
        }
        true
    }
}

/// Validators are heavy to initialize, so we want to re-use them when possible.
static VALIDATOR_CACHE: Lazy<DashMap<String, Arc<Validator>>> = Lazy::new(DashMap::new);
fn get_validator(
    json_path: &JsonPath,
    specification: &Value,
) -> Result<Arc<Validator>, OpenApiValidationError> {
    let string_path = json_path.format_path();
    if let Some(contents) = VALIDATOR_CACHE.get(&string_path) {
        return Ok(contents.clone());
    }
    let validator =
        match ValidatorFactory::build_validator_for_path(string_path, specification.clone()) {
            Ok(v) => Arc::new(v),
            Err(e) => return Err(e),
        };
    VALIDATOR_CACHE.insert(json_path.format_path(), validator.clone());
    Ok(validator)
}

pub(crate) struct ValidatorFactory;
impl ValidatorFactory {
    pub fn build_validator_for_path(
        json_path: String,
        specification: Value,
    ) -> Result<Validator, OpenApiValidationError> {
        let full_pointer_path = format!("@@root#/{}", json_path);
        let schema = json!({
            "$ref": full_pointer_path
        });

        let resource = match Resource::from_contents(specification) {
            Ok(res) => res,
            Err(_) => {
                return Err(OpenApiValidationError::InvalidSchema(
                    "Invalid specification provided".to_string(),
                ));
            }
        };
        let validator = match Validator::options()
            .with_resource("@@inner", resource)
            .build(&schema)
        {
            Ok(validator) => validator,
            Err(_) => {
                return Err(OpenApiValidationError::InvalidPath(
                    "Invalid json path provided".to_string(),
                ));
            }
        };
        Ok(validator)
    }
}

#[cfg(test)]
mod test {
    use crate::OpenApiValidator;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use oas3::spec::{ObjectOrReference, ObjectSchema, SchemaType, SchemaTypeSet};

    #[test]
    fn test_post_validation() {
        let test_request_path = "/pet";
        let test_request_method = "POST";
        let mut test_request_headers: HashMap<String, String> = HashMap::new();
        test_request_headers.insert("Accept".to_string(), "application/json".to_string());
        test_request_headers.insert("Content-Type".to_string(), "application/json".to_string());

        let post_body = json!({
            "id": 1,
            "category": {
              "id": 1,
              "name": "cat"
            },
            "name": "fluffy",
            "photoUrls": [
              "http://example.com/path/to/cat/1.jpg",
              "http://example.com/path/to/cat/2.jpg"
            ],
            "tags": [
              {
                "id": 1,
                "name": "cat"
              }
            ],
            "status": "available"
        });

        let mut validator = OpenApiValidator::from_file("test/openapi.json");
        let result = validator.validate_request(
            test_request_path,
            test_request_method,
            Some(&post_body),
            Some(&test_request_headers),
            None::<&HashMap<String, String>>,
        );
        assert!(result.is_ok());

        let invalid_post_body = json!({
            "id": 1,
            "category": {
              "id": 1,
              "name": "cat"
            },
            "name": "fluffy",
            "invalid_field": [
              "http://example.com/path/to/cat/1.jpg",
              "http://example.com/path/to/cat/2.jpg"
            ],
            "tags": [
              {
                "id": 1,
                "name": "cat"
              }
            ],
            "status": "available"
        });
        let result = validator.validate_request(
            test_request_path,
            test_request_method,
            Some(&invalid_post_body),
            Some(&test_request_headers),
            None::<&HashMap<String, String>>,
        );
        assert!(!result.is_ok());
    }

    #[test]
    fn test_get_validation() {
        let file = fs::read_to_string("test/openapi.json").unwrap();
        let test_request_path = "/pet/findById/123";
        let test_request_method = "GET";
        let mut test_request_headers: HashMap<String, String> = HashMap::new();
        test_request_headers.insert("Accept".to_string(), "application/json".to_string());

        let mut validator = OpenApiValidator::from_json_string(file);
        let result = validator.validate_request(
            test_request_path,
            test_request_method,
            None,
            Some(&test_request_headers),
            None::<&HashMap<String, String>>,
        );

        assert!(result.is_ok());
    }

    /// Example schema taken from: https://swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not/
    #[test]
    fn test_validate_object_rules_all_of() {
        //let file = fs::read_to_string("test/openapi.json").unwrap();

        let mut pet_type_schema = ObjectSchema::default();
        pet_type_schema.schema_type = Some(SchemaTypeSet::Single(SchemaType::String));

        let mut pet_props = ObjectSchema::default();
        pet_props.schema_type = Some(SchemaTypeSet::Single(SchemaType::Object));
        pet_props.required = vec!["pet_type".to_string()];
        pet_props.properties.insert(
            "pet_type".to_string(),
            ObjectOrReference::Object(pet_type_schema),
        );

        let mut cat_hunting_schema = ObjectSchema::default();
        cat_hunting_schema.schema_type = Some(SchemaTypeSet::Single(SchemaType::Boolean));

        let mut cat_age_schema = ObjectSchema::default();
        cat_age_schema.schema_type = Some(SchemaTypeSet::Single(SchemaType::Integer));

        let mut cat_props = ObjectSchema::default();
        cat_props.schema_type = Some(SchemaTypeSet::Single(SchemaType::Object));
        cat_props.required = vec!["hunts".to_string(), "age".to_string()];
        cat_props.properties.insert(
            "hunts".to_string(),
            ObjectOrReference::Object(cat_hunting_schema),
        );
        cat_props
            .properties
            .insert("age".to_string(), ObjectOrReference::Object(cat_age_schema));

        let mut cat_schema = ObjectSchema::default();
        cat_schema.all_of.push(ObjectOrReference::Object(pet_props));
        cat_schema.all_of.push(ObjectOrReference::Object(cat_props));

        // has pet_type, and all cat schema props
        let valid_request_body = json!({
            "pet_type": "Cat",
            "hunts": true,
            "age": 9
        });

        assert!(OpenApiValidator::validate_with_schema(&valid_request_body, &cat_schema).is_ok());

        // Missing pet_type
        let invalid_request_body = json!({
            "age": 3,
            "hunts": true
        });

        assert!(
            !OpenApiValidator::validate_with_schema(&invalid_request_body, &cat_schema).is_ok()
        );

        // Cat schema does not have 'bark' property, but additional properties are allowed
        let invalid_request_body = json!({
            "pet_type": "Cat",
            "age": 3,
            "hunts": true,
            "bark": true
        });

        assert!(OpenApiValidator::validate_with_schema(&invalid_request_body, &cat_schema).is_ok());
    }
}
