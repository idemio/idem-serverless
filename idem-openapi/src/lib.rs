use idem_config::config_cache::get_file;
use oas3::OpenApiV3Spec;
use oas3::spec::{ObjectOrReference, ObjectSchema, Operation, Parameter, PathItem, SchemaType, SchemaTypeSet};
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct OpenApiValidator {
    specification: OpenApiV3Spec,
}

const EMAIL_REGEX: &str = r#"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$"#;
const DATE_TIME_RFC3339_REGEX: &str =
    r#"^((?:(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2}(?:\.\d+)?))(Z|[\+-]\d{2}:\d{2})?)$"#;
const DATE_RFC339_REGEX: &str = r#"^(\d{4}-\d{2}-\d{2})$"#;
const UUID_REGEX: &str = r#"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"#;
const IPV6_REGEX: &str = r#"^((?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4})$"#;

// technically, this regex allows you to have an invalid ip. i.e. 999.999.999.999.
// is it worth adding more constraints?
const IPV4_REGEX: &str = r#"^((?:[0-9]{1,3}\.){3}[0-9]{1,3})$"#;
const HOSTNAME_REGEX: &str = r#"^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$"#;

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

    fn get_param_value(
        &self,
        param_name: &str,
        endpoint_params: &Vec<ObjectOrReference<Parameter>>,
    ) -> Option<Parameter> {
        for param_or_ref in endpoint_params {
            match param_or_ref.resolve(&self.specification) {
                Ok(param) => {
                    if param.name == param_name {
                        return Some(param.clone());
                    }
                }
                Err(_) => continue,
            }
        }
        None
    }

    fn validate_string_format(string_value: &str, format: &str) -> bool {
        match format {
            "date" => {
                if let Ok(date_regex) = regex::Regex::new(DATE_RFC339_REGEX) {
                    date_regex.is_match(string_value)
                } else {
                    false
                }
            }
            "date-time" => {
                if let Ok(date_time_regex) = regex::Regex::new(DATE_TIME_RFC3339_REGEX) {
                    date_time_regex.is_match(string_value)
                } else {
                    false
                }
            }
            "email" => {
                if let Ok(email_regex) = regex::Regex::new(EMAIL_REGEX) {
                    email_regex.is_match(string_value)
                } else {
                    false
                }
            }
            "ipv4" => {
                if let Ok(ipv4_regex) = regex::Regex::new(IPV4_REGEX) {
                    ipv4_regex.is_match(string_value)
                } else {
                    false
                }
            }
            "ipv6" => {
                if let Ok(ipv6_regex) = regex::Regex::new(IPV6_REGEX) {
                    ipv6_regex.is_match(string_value)
                } else {
                    false
                }
            }
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
                if *min > str_val.len() as u64 {}
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

    fn validate_integer_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(int_val) = value.as_i64() {
            let int_val: i64 = int_val;
            if let Some(max_int) = schema.maximum.as_ref().and_then(|v| v.as_i64()) {
                if int_val > max_int {
                    return false;
                }
            }

            if let Some(min_int) = schema.minimum.as_ref().and_then(|v| v.as_i64()) {
                if int_val < min_int {
                    return false;
                }
            }

            if let Some(multiple_of) = schema.multiple_of.as_ref().and_then(|v| v.as_i64()) {
                if int_val % multiple_of != 0 {
                    return false;
                }
            }

            if let Some(format) = &schema.format {
                if !Self::validate_number_format(value, format) {
                    return false;
                }
            }
        }
        true
    }

    fn validate_for_type(value: &Value, type_val: &SchemaType, schema: &ObjectSchema) -> bool {
        match type_val {
            SchemaType::Boolean => todo!("Implement boolean validation"),
            SchemaType::Integer => Self::validate_integer_rules(value, schema),
            SchemaType::Number => todo!("Implement number validation"),
            SchemaType::String => Self::validate_string_rules(value, schema),
            SchemaType::Array => todo!("implement array validation"),
            SchemaType::Object => todo!("Implement object validation"),
            SchemaType::Null => true,
        }
    }

    fn validate_with_schema(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(schema_type_set) = &schema.schema_type {
            return match schema_type_set {
                /* validate single types */
                SchemaTypeSet::Single(type_val) => Self::validate_for_type(value, type_val, schema),

                /* validate for multiple types */
                // TODO - this assumes anyOf (need to implement to handle oneOf and allOf)
                SchemaTypeSet::Multiple(type_vals) => type_vals
                    .iter()
                    .any(|type_val| Self::validate_for_type(value, type_val, schema)),
            };
        }
        false
    }

    pub fn get_scopes_for_path(&self, request_path: &str, method: &str) -> Vec<String> {
        if let Some(spec_path) = self.get_matching_operation(request_path, method) {
            todo!("Operation in the oas3 crate repo does not have security schemas")
        }
        vec![]
    }

    pub fn get_matching_operation(
        &self,
        path_to_match: &str,
        method_to_match: &str,
    ) -> Option<&Operation> {
        let spec_paths = match &self.specification.paths {
            Some(paths) => paths,
            None => todo!("Handle no paths being present in the specification"),
        };

        for (spec_path, path_item) in spec_paths.iter() {
            if let Some((_, op)) = path_item
                .methods()
                .into_iter()
                .find(|(method, _)| method.as_str() == method_to_match)
            {
                let path_method_item_params = &op.parameters;
                if self.match_path_segment(path_to_match, spec_path, path_method_item_params) {
                    return Some(op);
                }
            }
        }
        None
    }

    fn match_path_segment(
        &self,
        target_path: &str,
        spec_path: &str,
        path_method_item_params: &Vec<ObjectOrReference<Parameter>>,
    ) -> bool {
        let mut matching_segments = 0usize;
        let mut segment_count = 0usize;
        spec_path
            .split(Self::PATH_SPLIT)
            .collect::<Vec<&str>>()
            .iter()
            .zip(target_path.split(Self::PATH_SPLIT).collect::<Vec<&str>>().iter())
            .for_each(|(spec_segment, target_segment): (&&str, &&str)| {
                segment_count += 1;

                /* if the segment in the spec is a path parameter, we have to validate to make sure the target path given matches the schema for that parameter name */
                if let (Some(start), Some(end)) = (spec_segment.find('{'), spec_segment.find('}')) {
                    let spec_segment_param_name = &spec_segment[start + 1..end];
                    if let Some(spec_segment_param_value) =
                        self.get_param_value(spec_segment_param_name, path_method_item_params)
                    {
                        if let Some(resolved_schema) = spec_segment_param_value.schema {
                            if let Ok(schema) = resolved_schema.resolve(&self.specification) {
                                if Self::validate_with_schema(&json!(target_segment), &schema) {
                                    matching_segments += 1;
                                }
                            }
                        };
                    }

                /* simplest case, check if the string values are the same. */
                } else if spec_segment == target_segment {
                    matching_segments += 1;
                }
            });

        /* if all segments match, we found our path */
        if matching_segments == segment_count {
            return true;
        }

        false
    }

    pub fn validate_request(
        &self,
        path: &str,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&Value>,
    ) -> Result<(), ()> {
        let matching_path = self.get_matching_operation(path, method);
        if matching_path.is_none() {
            return Err(());
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::OpenApiValidator;
    use oas3::spec::ObjectSchema;
    use serde_json::{Number, json};
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_read_spec() {
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
            &test_request_headers,
            None,
        );
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
}
