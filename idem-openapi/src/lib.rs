mod json_path_builder;

use crate::json_path_builder::JsonPointerPathBuilder;
use jsonschema::Validator;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, SchemaType, SchemaTypeSet,
};
use oas3::{OpenApiV3Spec, Spec};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;

pub struct OpenApiValidator {
    specification: OpenApiV3Spec,
    root_schema: Value,
    cached_validators: HashMap<String, Validator>,
}

impl OpenApiValidator {
    const PATH_SPLIT: char = '/';
    const PATH_PARAM_LEFT: char = '{';
    const PATH_PARAM_RIGHT: char = '}';

    pub fn from_file(specification_filename: &str) -> Self {
        let file = fs::read_to_string(specification_filename).unwrap();
        let mut spec: Value = serde_json::from_str(&file).unwrap();
        spec["$id"] = json!("@@root");
        let traversable_spec = oas3::from_json(file).unwrap();
        Self {
            specification: traversable_spec,
            root_schema: spec,
            cached_validators: HashMap::new(),
        }
    }

    fn object_schema_to_value(schema: &ObjectSchema) -> Result<Value, ()> {
        match serde_json::to_value(schema) {
            Ok(val) => Ok(val),
            Err(_) => Err(()),
        }
    }

    fn validate_with_schema(value: &Value, schema: &ObjectSchema) -> Result<(), ()> {
        let schema_as_value = Self::object_schema_to_value(schema)?;
        match jsonschema::validate(&schema_as_value, value) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn find_matching_operation(
        &self,
        path_to_match: &str,
        method_to_match: &str,
    ) -> Option<(&Operation, JsonPointerPathBuilder)> {
        let spec_paths = match &self.specification.paths {
            Some(paths) => paths,
            None => return None,
        };

        for (spec_path, path_item) in spec_paths.iter() {
            if let Some((_, op)) = path_item
                .methods()
                .into_iter()
                .find(|(method, _)| method.as_str() == method_to_match)
            {
                let path_method_item_params = &op.parameters;
                if self.match_path_segments(path_to_match, spec_path, path_method_item_params) {
                    let mut json_path_builder = JsonPointerPathBuilder::new();
                    json_path_builder
                        .add_segment("paths".to_string())
                        .add_segment(spec_path.to_string())
                        .add_segment(method_to_match.to_lowercase().to_string());
                    return Some((op, json_path_builder));
                }
            }
        }
        None
    }

    fn validate_path_param(
        &self,
        param_name: &str,
        target_segment: &str,
        path_method_item_params: &Vec<ObjectOrReference<Parameter>>,
    ) -> Result<(), ()> {
        if let Some(param) = self.get_parameter_from_name(param_name, path_method_item_params) {
            if let Some(resolved_schema) = param
                .schema
                .and_then(|schema| schema.resolve(&self.specification).ok())
            {
                if let Ok(value) =
                    Self::try_cast_path_param_to_schema_type(target_segment, &resolved_schema)
                {
                    let value = &value;
                    let res = Self::validate_with_schema(value, &resolved_schema);
                    return res;
                }
            }
        }
        Ok(())
    }

    fn try_cast_to_type(target_segment: &str, schema_type: &SchemaType) -> Result<Value, ()> {
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

    fn try_cast_path_param_to_schema_type(
        target_segment: &str,
        schema: &ObjectSchema,
    ) -> Result<Value, ()> {
        let param_type = schema.schema_type.as_ref().unwrap();
        match param_type {
            SchemaTypeSet::Single(single) => Self::try_cast_to_type(target_segment, &single),
            SchemaTypeSet::Multiple(multi) => {
                for m_type in multi {
                    let res = Self::try_cast_to_type(target_segment, &m_type);
                    if res.is_ok() {
                        return res;
                    }
                }
                Err(())
            }
        }
    }

    fn get_parameter_from_name(
        &self,
        param_name: &str,
        endpoint_params: &Vec<ObjectOrReference<Parameter>>,
    ) -> Option<Parameter> {
        // look through each parameter for the operation, if the 'name' field value
        // matches the provided 'param_name' then return that.
        // returns None if there is no matching parameter schema for the operation.
        endpoint_params.iter().find_map(|param| {
            param.resolve(&self.specification).ok().and_then(|param| {
                if param_name == param.name.as_str() {
                    Some(param.clone())
                } else {
                    None
                }
            })
        })
    }

    fn match_path_segments(
        &self,
        target_path: &str,
        spec_path: &str,
        path_method_item_params: &Vec<ObjectOrReference<Parameter>>,
    ) -> bool {
        let target_segments = target_path.split(Self::PATH_SPLIT).collect::<Vec<&str>>();
        let spec_segments = spec_path.split(Self::PATH_SPLIT).collect::<Vec<&str>>();

        // The number of segments in the path, and the number of segments that match the given path.
        // if the numbers are equal, it means we've found a match.
        let (matching_segments, segment_count) =
            spec_segments.iter().zip(target_segments.iter()).fold(
                (0, 0),
                |(mut matches, mut count), (spec_segment, target_segment)| {
                    count += 1;

                    // If the path in the spec contains a path parameter,
                    // we need to make sure the value in the given_path's value at the segment
                    // follows the schema rules defined in the specification.
                    // If the validation fails, we do not consider it a match.
                    if let Some(param_name) = Self::extract_path_param_name(spec_segment) {
                        match self.validate_path_param(
                            param_name,
                            target_segment,
                            path_method_item_params,
                        ) {
                            Ok(_) => matches += 1,
                            Err(_) => return (matches, count),
                        }

                    // Simplest case where we check to see if the segment values are the same (non-path parameter)
                    } else if spec_segment == target_segment {
                        matches += 1;
                    }

                    (matches, count)
                },
            );

        matching_segments == segment_count
    }

    /// Extracts the path parameter name (between the chars '{' and '}')
    /// returns None if there is no path parameter in the segment.
    fn extract_path_param_name(segment: &str) -> Option<&str> {
        segment.find(Self::PATH_PARAM_LEFT).and_then(|start| {
            segment
                .find(Self::PATH_PARAM_RIGHT)
                .map(|end| &segment[start + 1..end])
        })
    }

    fn validate_schema_from_pointer(
        &mut self,
        instance: &Value,
        pointer_path: String,
    ) -> Result<(), ()> {
        let full_pointer_path = format!("@@root#/{}", pointer_path);
        if let Some(validator) = self.cached_validators.get(&full_pointer_path) {
            println!("Using cached validator for path: {}", full_pointer_path);
            match validator.validate(instance) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        } else {
            let schema = json!({
                "$ref": full_pointer_path
            });

            let validator = Validator::options()
                .with_resource(
                    "root_id",
                    jsonschema::Resource::from_contents(self.root_schema.clone())
                        .expect("failed to load spec"),
                )
                .build(&schema)
                .expect("failed to build validator");
            let res = validator.validate(instance);
            self.cached_validators.insert(full_pointer_path, validator);
            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }

    pub fn resolve_request_body(
        &self,
        operation: &Operation,
        content_type: &str,
        json_path: &mut JsonPointerPathBuilder,
    ) -> Option<ObjectSchema> {
        let request_body_ref = operation
            .request_body
            .as_ref()?
            .resolve(&self.specification)
            .ok()?;
        json_path.add_segment("requestBody".to_string());

        let content = request_body_ref.content.get(content_type)?;
        json_path.add_segment("content".to_string());
        json_path.add_segment(content_type.to_string());

        let schema = content.schema.as_ref()?.resolve(&self.specification).ok()?;
        json_path.add_segment("schema".to_string());

        Some(schema)
    }

    //    pub fn validate_instance_at_path(
    //        &mut self,
    //        instance: &Value,
    //        json_path: JsonPointerPathBuilder,
    //    ) -> Result<(), ()> {
    //        self.validate_schema_from_pointer(instance, json_path.build())
    //    }

    pub fn validate_request_headers() {
        todo!()
    }

    pub fn validate_request_query_params() {
        todo!()
    }

    pub fn validate_request_body(
        &mut self,
        path: &str,
        method: &str,
        content_type_header: &str,
        body: &Value,
    ) -> Result<(), ()> {
        let (operation, mut json_path) = self.find_matching_operation(path, method).unwrap();

        // TODO - come up with a better way to resolve paths and objects. The current way is a little clunky.
        let _ = self
            .resolve_request_body(operation, content_type_header, &mut json_path)
            .ok_or_else(|| "Failed to resolve the request body schema".to_string());

        self.validate_schema_from_pointer(body, json_path.build())
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

#[cfg(test)]
mod test {
    use crate::OpenApiValidator;
    use oas3::spec::{ObjectOrReference, ObjectSchema, SchemaType, SchemaTypeSet};
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;

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
        let result = validator.validate_request_body(
            test_request_path,
            test_request_method,
            "application/json",
            &post_body,
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
        let result = validator.validate_request_body(
            test_request_path,
            test_request_method,
            "application/json",
            &invalid_post_body,
        );
        assert!(!result.is_ok());
    }

    //    #[test]
    //    fn test_get_validation() {
    //        let file = fs::read_to_string("test/openapi.json").unwrap();
    //        let spec = oas3::from_json(file).unwrap();
    //        let test_request_path = "/pet/findById/123";
    //        let test_request_method = "GET";
    //        let mut test_request_headers: HashMap<String, String> = HashMap::new();
    //        test_request_headers.insert("Accept".to_string(), "application/json".to_string());
    //
    //        let validator = OpenApiValidator::from_spec(spec);
    //        let result = validator.validate_request(
    //            test_request_path,
    //            test_request_method,
    //            Some(&test_request_headers),
    //            None,
    //            None,
    //        );
    //
    //        assert!(result.is_ok());
    //    }

    /// Example schema taken from: https://swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not/
    #[test]
    fn test_validate_object_rules_all_of() {
        let file = fs::read_to_string("test/openapi.json").unwrap();

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
            "hunts": "test",
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

        // Cat schema does not have 'bark' property
        let invalid_request_body = json!({
            "pet_type": "Cat",
            "age": 3,
            "hunts": true,
            "bark": true
        });

        assert!(
            !OpenApiValidator::validate_with_schema(&invalid_request_body, &cat_schema).is_ok()
        );
    }
}
