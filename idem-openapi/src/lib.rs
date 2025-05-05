mod constants;

use idem_config::config_cache::get_file;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, RequestBody, SchemaType,
    SchemaTypeSet,
};
use oas3::{OpenApiV3Spec, Spec};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

pub struct OpenApiValidator {
    specification: OpenApiV3Spec,
}

impl OpenApiValidator {
    const PATH_SPLIT: char = '/';
    const PATH_PARAM_LEFT: char = '{';
    const PATH_PARAM_RIGHT: char = '}';

    pub fn from_file(specification_filename: &str) -> Result<Self, ()> {
        match get_file(specification_filename) {
            Ok(file) => {
                let file: &str = &file;
                match oas3::from_json(file) {
                    Ok(spec) => Ok(Self {
                        specification: spec,
                    }),
                    Err(_) => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    }

    pub fn from_spec(specification: OpenApiV3Spec) -> Self {
        Self { specification }
    }

    fn validate_for_type(
        value: &Value,
        type_val: &SchemaType,
        schema: &ObjectSchema,
        spec: &Spec,
    ) -> bool {
        match type_val {
            SchemaType::Boolean => Self::validate_boolean_rules(value, schema),
            SchemaType::Integer => Self::validate_integer_rules(value, schema),
            SchemaType::Number => Self::validate_number_rules(value, schema),
            SchemaType::String => Self::validate_string_rules(value, schema),
            SchemaType::Array => Self::validate_array_rules(value, schema, spec),
            SchemaType::Object => Self::validate_object_rules(value, schema, spec),
            SchemaType::Null => Self::validate_null_rules(value, schema),
        }
    }

    pub fn validate_string_format(string_value: &str, format: &str) -> bool {
        match format {
            constants::format::DATE => regex::Regex::new(constants::pattern::DATE_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            constants::format::DATE_TIME => regex::Regex::new(constants::pattern::DATE_TIME_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            constants::format::EMAIL => regex::Regex::new(constants::pattern::EMAIL_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            constants::format::IPV4 => regex::Regex::new(constants::pattern::IPV4_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            constants::format::IPV6 => regex::Regex::new(constants::pattern::IPV6_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            constants::format::UUID => regex::Regex::new(constants::pattern::UUID_REGEX)
                .is_ok_and(|reg| reg.is_match(string_value)),
            _ => {
                println!("Unknown format: {}", format);
                false
            }
        }
    }

    fn validate_number_format(number_value: &Value, format: &str) -> bool {
        todo!("implement number formats!")
    }

    fn validate_string_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(str_val) = value.as_str() {
            if let Some(max) = &schema.max_length {
                if *max < str_val.len() as u64 {
                    return false;
                }
            }

            if let Some(min) = &schema.min_length {
                if *min > str_val.len() as u64 {
                    return false;
                }
            }

            if let Some(pattern) = &schema.pattern {
                if let Ok(pattern) = regex::Regex::new(pattern) {
                    if !pattern.is_match(str_val) {
                        return false;
                    }
                }
            }

            if let Some(format) = &schema.format {
                if !Self::validate_string_format(str_val, format) {
                    return false;
                }
            }

            return true;
        }
        false
    }

    fn value_is_in_enum(value: &Value, enum_values: &Vec<Value>) -> bool {
        enum_values.iter().any(|enum_value| enum_value == value)
    }

    fn validate_integer_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(int_val) = value.as_i64() {
            if !schema.enum_values.is_empty() {
                if !Self::value_is_in_enum(value, &schema.enum_values) {
                    return false;
                }
            }

            if schema
                .maximum
                .as_ref()
                .and_then(|v| v.as_i64())
                .map_or(false, |max_int| int_val > max_int)
            {
                return false;
            }

            if schema
                .minimum
                .as_ref()
                .and_then(|v| v.as_i64())
                .map_or(false, |min_int| int_val < min_int)
            {
                return false;
            }

            if schema
                .exclusive_maximum
                .as_ref()
                .and_then(|v| v.as_i64())
                .map_or(false, |exclusive_max| int_val >= exclusive_max)
            {
                return false;
            }

            if schema
                .exclusive_minimum
                .as_ref()
                .and_then(|v| v.as_i64())
                .map_or(false, |exclusive_min| int_val <= exclusive_min)
            {
                return false;
            }

            if schema
                .multiple_of
                .as_ref()
                .and_then(|v| v.as_i64())
                .map_or(false, |multiple_0f| int_val % multiple_0f != 0)
            {
                return false;
            }

            if let Some(format) = &schema.format {
                if !Self::validate_number_format(value, format) {
                    return false;
                }
            }

            return true;
        }
        false
    }

    fn validate_array_rules(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        if let Some(array_val) = value.as_array() {
            if !schema.enum_values.is_empty() {
                if !Self::value_is_in_enum(value, &schema.enum_values) {
                    return false;
                }
            }

            if schema
                .max_items
                .is_some_and(|max| array_val.len() > max as usize)
            {
                return false;
            }

            if schema
                .min_items
                .is_some_and(|min| array_val.len() < min as usize)
            {
                return false;
            }

            if let Some(unique_items) = schema.unique_items {
                if unique_items {
                    let mut found_set: HashSet<&Value> = HashSet::new();
                    for item in array_val {
                        if !found_set.insert(item) {
                            return false;
                        }
                    }
                }
            }

            if let Some(item_schema) = &schema.items {
                if let Ok(resolved) = item_schema.resolve(spec) {
                    for array_item in array_val {
                        if !Self::validate_with_schema(array_item, &resolved, spec) {
                            return false;
                        }
                    }
                }
            }

            return true;
        }
        false
    }

    fn validate_boolean_rules(value: &Value, _schema: &ObjectSchema) -> bool {
        // TODO - Are there boolean rules?
        if let Some(_) = value.as_bool() {
            return true;
        }
        false
    }

    fn validate_null_rules(value: &Value, _schema: &ObjectSchema) -> bool {
        // TODO - Are there 'null' rules?
        if let Some(_) = value.as_null() {
            return true;
        }
        false
    }

    fn validate_number_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(number_val) = value.as_f64() {
            if !schema.enum_values.is_empty() {
                if !Self::value_is_in_enum(value, &schema.enum_values) {
                    return false;
                }
            }

            if schema
                .maximum
                .as_ref()
                .and_then(|v| v.as_f64())
                .map_or(false, |max| number_val > max)
            {
                return false;
            }

            if schema
                .minimum
                .as_ref()
                .and_then(|v| v.as_f64())
                .map_or(false, |min| number_val < min)
            {
                return false;
            }

            if schema
                .exclusive_maximum
                .as_ref()
                .and_then(|v| v.as_f64())
                .map_or(false, |ex_max| number_val >= ex_max)
            {
                return false;
            }

            if schema
                .exclusive_minimum
                .as_ref()
                .and_then(|v| v.as_f64())
                .map_or(false, |ex_min| number_val <= ex_min)
            {
                return false;
            }

            return true;
        }
        false
    }

    fn validate_object_rules(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        if let Some(object_val) = value.as_object() {
            if !schema.enum_values.is_empty() {
                if !Self::value_is_in_enum(value, &schema.enum_values) {
                    return false;
                }
            }

            if !schema.required.is_empty() {
                let required_fields = &schema.required;
                if required_fields
                    .iter()
                    .any(|field| !object_val.contains_key(field))
                {
                    return false;
                }
            }

            for (field, field_schema) in &schema.properties {
                if let (Ok(resolved_schema), Some(object_field_val)) =
                    (field_schema.resolve(&spec), object_val.get(field))
                {
                    if !Self::validate_with_schema(object_field_val, &resolved_schema, spec) {
                        return false;
                    }
                }
            }
            return true;
        }
        false
    }

    fn validate_all_of_schema_set(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        let mut valid_count = 0usize;
        let goal_matches = schema.all_of.len();
        for all_of_schema in &schema.all_of {
            if all_of_schema
                .resolve(&spec)
                .is_ok_and(|schema| Self::validate_with_schema(value, &schema, spec))
            {
                valid_count += 1;
            }
        }

        // all matches
        valid_count == goal_matches
    }

    fn validate_any_of_schema_set(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        let mut valid_count = 0usize;
        for any_of_schema in &schema.any_of {
            if any_of_schema
                .resolve(&spec)
                .is_ok_and(|schema| Self::validate_with_schema(value, &schema, spec))
            {
                valid_count += 1;
            }
        }

        // any matches
        valid_count > 0
    }

    fn validate_one_of_schema_set(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        let mut valid_count = 0usize;
        for any_of_schema in &schema.one_of {
            if any_of_schema
                .resolve(&spec)
                .is_ok_and(|schema| Self::validate_with_schema(value, &schema, spec))
            {
                valid_count += 1;
            }
        }

        // one and only one match
        valid_count == 1
    }

    pub fn validate_with_schema(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        // Default case where we have a type field present
        if let Some(schema_type_set) = &schema.schema_type {
            return match schema_type_set {
                // validate single types
                SchemaTypeSet::Single(type_val) => {
                    Self::validate_for_type(value, type_val, schema, spec)
                }

                // validate for multiple types
                // TODO - Look into this
                SchemaTypeSet::Multiple(type_vals) => type_vals
                    .iter()
                    .any(|type_val| Self::validate_for_type(value, type_val, schema, spec)),
            };

        // Check if for oneOf, anyOf, allOf, and not
        // NOTE: oas3-rs has not implemented 'not' yet
        } else if !schema.one_of.is_empty()
            || !schema.all_of.is_empty()
            || !schema.any_of.is_empty()
        /* !schema.not.is_empty() */
        {
            if !schema.all_of.is_empty() {
                return Self::validate_all_of_schema_set(value, schema, spec);
            } else if !schema.any_of.is_empty() {
                return Self::validate_any_of_schema_set(value, schema, spec);
            } else if !schema.one_of.is_empty() {
                return Self::validate_one_of_schema_set(value, schema, spec);
            }

            /* handle 'not' when implemented */
        }

        false
    }

    pub fn get_security_scopes(
        &self,
        request_path: &str,
        method: &str,
    ) -> Option<HashMap<String, Vec<String>>> {
        if let Some(operation) = self.get_matching_operation(request_path, method) {
            if !operation.security.is_empty() {
                let mut scopes_for_path = HashMap::new();
                let schemes = &operation.security;

                // TODO - handle multiple flows, for now we only grab the first security requirement definition for the endpoint.
                let security_scheme = &schemes.get(0).unwrap().0;
                for (security_scheme_name, security_scheme_value) in security_scheme {
                    scopes_for_path.insert(
                        security_scheme_name.to_string(),
                        security_scheme_value.to_vec(),
                    );
                }

                return Some(scopes_for_path);
            }
        }
        None
    }

    pub fn get_matching_operation(
        &self,
        path_to_match: &str,
        method_to_match: &str,
    ) -> Option<&Operation> {
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
                    return Some(op);
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
    ) -> bool {
        if let Some(param) = self.get_parameter_from_name(param_name, path_method_item_params) {
            if let Some(resolved_schema) = param
                .schema
                .and_then(|schema| schema.resolve(&self.specification).ok())
            {
                if let Ok(value) = Self::try_cast_path_param_to_schema_type(target_segment, &resolved_schema) {
                    return Self::validate_with_schema(&value, &resolved_schema, &self.specification);
                }
            }
        }
        false
    }

    fn try_cast_path_param_to_schema_type(target_segment: &str, schema: &ObjectSchema) -> Result<Value, ()> {
        let param_type = schema.schema_type.as_ref().unwrap();
        match param_type {
            SchemaTypeSet::Single(single) => {

                // TODO - DRY this
                match single {
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
                    SchemaType::String => {
                        Ok(json!(target_segment))
                    }

                    // invalid type for path parameter
                    |_ => Err(()),
                }
            }
            SchemaTypeSet::Multiple(multi) => {
                for m_type in multi {
                    let res = match m_type {
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
                        SchemaType::String => {
                            Ok(json!(target_segment))
                        }
                        |_ => Err(()),
                    };

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
                        if self.validate_path_param(
                            param_name,
                            target_segment,
                            path_method_item_params,
                        ) {
                            matches += 1;
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

    fn validate_request_body(
        body: &Value,
        request_schema: &RequestBody,
        headers: &HashMap<String, String>,
        spec: &Spec,
    ) -> bool {
        if let Some((_, content_type)) = headers
            .iter()
            .find(|(header_name, _)| header_name.to_lowercase() == "content-type")
        {
            if let Some(data_type) =
                content_type
                    .split(';')
                    .collect::<Vec<&str>>()
                    .iter()
                    .find(|segment| {
                        segment.starts_with("application")
                            || segment.starts_with("multipart")
                            || segment.starts_with("text")
                    })
            {
                if let Some(matching_schema) = request_schema
                    .content
                    .get(*data_type)
                    .and_then(|content| content.schema.as_ref())
                {
                    if let Ok(resolved_schema) = matching_schema.resolve(&spec) {
                        return Self::validate_with_schema(body, &resolved_schema, spec);
                    }
                }
            }
        }

        false
    }

    pub fn validate_request(
        &self,
        path: &str,
        method: &str,
        headers: Option<&HashMap<String, String>>,
        query_params: Option<&HashMap<String, String>>,
        body: Option<&Value>,
    ) -> Result<(), ()> {
        if let Some(operation) = self.get_matching_operation(path, method) {
            /* validate body */
            if let (Some(request_schema), Some(body), Some(headers)) =
                (&operation.request_body, body, headers)
            {
                if let Ok(resolved_schema) = request_schema.resolve(&self.specification) {
                    if !Self::validate_request_body(
                        body,
                        &resolved_schema,
                        headers,
                        &self.specification,
                    ) {
                        return Err(());
                    }
                }
            }

            /* validate parameters */
            if !operation.parameters.is_empty() {
                let parameters = &operation.parameters;

                /* validate headers parameters if present */
                if headers.is_some_and(|headers| {
                    !Self::validate_parameter_type(
                        headers,
                        ParameterIn::Header,
                        parameters,
                        &self.specification,
                    )
                }) {
                    return Err(());
                }

                /* validate query parameters if present */
                if query_params.is_some_and(|query_params| {
                    !Self::validate_parameter_type(
                        query_params,
                        ParameterIn::Query,
                        &parameters,
                        &self.specification,
                    )
                }) {
                    return Err(());
                }
            }

            return Ok(());
        }

        Err(())
    }

    fn validate_parameter_type(
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

        Self::validate_given_parameters(given_parameters, &relevant_parameters, &spec)
    }

    fn validate_given_parameters(
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
                        // TODO - can content be defined in a parameter?
                        todo!("validating 'content' field in parameters not implemented yet")
                    } else if let Some(schema) = resolved_param.schema {
                        if let Ok(schema) = schema.resolve(&specification) {
                            if !Self::validate_with_schema(
                                &Value::String(param_value.clone()),
                                &schema,
                                &specification,
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
    use serde_json::{Number, json};
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_request_validation() {
        let file = fs::read_to_string("test/openapi.json").unwrap();
        let spec = oas3::from_json(file).unwrap();
        let test_request_path = "/pet/findById/123";
        let test_request_method = "GET";
        let mut test_request_headers: HashMap<String, String> = HashMap::new();
        test_request_headers.insert("Accept".to_string(), "application/json".to_string());

        let validator = OpenApiValidator::from_spec(spec);
        let result = validator.validate_request(
            test_request_path,
            test_request_method,
            Some(&test_request_headers),
            None,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integer_rules() {
        let mut schema = ObjectSchema::default();
        schema.maximum = Some(Number::from(10));
        schema.minimum = Some(Number::from(5));
        schema.multiple_of = Some(Number::from(2));

        // > 5 < 10 multiple of 2
        let value = json!(8); // Should be valid
        assert!(OpenApiValidator::validate_integer_rules(&value, &schema));

        // > 5 > 10 multiple of 2
        let value = json!(12);
        assert!(!OpenApiValidator::validate_integer_rules(&value, &schema));

        // > 5 < 10 !multiple of 2
        let value = json!(7);
        assert!(!OpenApiValidator::validate_integer_rules(&value, &schema));

        // < 5 < 10 multiple of 2
        let value = json!(4);
        assert!(!OpenApiValidator::validate_integer_rules(&value, &schema));
    }

    /// Example schema taken from: https://swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not/
    #[test]
    fn test_validate_object_rules_all_of() {
        let file = fs::read_to_string("test/openapi.json").unwrap();
        let spec = oas3::from_json(file).unwrap();

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

        assert!(OpenApiValidator::validate_with_schema(
            &valid_request_body,
            &cat_schema,
            &spec
        ));

        // Missing pet_type
        let invalid_request_body = json!({
            "age": 3,
            "hunts": true
        });

        assert!(!OpenApiValidator::validate_with_schema(
            &invalid_request_body,
            &cat_schema,
            &spec
        ));

        //        // Cat schema does not have 'bark' property
        //        let invalid_request_body = json!({
        //            "pet_type": "Cat",
        //            "age": 3,
        //            "hunts": true,
        //            "bark": true
        //        });
        //
        //        assert!(!OpenApiValidator::validate_with_schema(
        //            &invalid_request_body,
        //            &cat_schema,
        //            &spec
        //        ));
    }
}
