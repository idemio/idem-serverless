mod constants;

use idem_config::config_cache::get_file;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, Parameter, ParameterIn, RequestBody, SchemaType,
    SchemaTypeSet,
};
use oas3::{OpenApiV3Spec, Spec};
use serde_json::{Value, json};
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

    /// Validates a JSON value against the specified schema type and related rules.
    ///
    /// This function dispatches the validation of a given value based on its type, utilizing
    /// specific validation functions for each schema type (e.g., boolean, integer, string, etc.).
    ///
    /// # Parameters
    /// - `value`: The JSON value to validate (`&Value`).
    /// - `type_val`: The schema type to validate against (`&SchemaType`).
    /// - `schema`: The `ObjectSchema` containing the constraints and rules for validation.
    /// - `spec`: The `Spec` used for resolving schema references and performing additional validation for complex types.
    ///
    /// # Returns
    /// - `true`: If the value matches the specified schema type and satisfies all associated rules.
    /// - `false`: If the value does not match the schema type or violates any associated rules.
    ///
    /// # Supported Schema Types
    /// - **Boolean**: Validates against boolean-specific rules.
    /// - **Integer**: Validates an integer value, including type and constraints checks.
    /// - **Number**: Validates a numeric value, including type and constraints checks.
    /// - **String**: Validates a string value, including type and format checks.
    /// - **Array**: Validates an array and its elements based on schema rules.
    /// - **Object**: Validates an object, including required fields and property validations.
    /// - **Null**: Validates if the value is null.
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

    /// Validates a string value against a specified format.
    ///
    /// This function checks if a given string matches one of the predefined formats such as
    /// date, date-time, email, IPv4, IPv6, or UUID, based on regular expression patterns.
    ///
    /// # Parameters
    /// - `string_value`: The string value to validate.
    /// - `format`: The format type to validate against (e.g., `date`, `date-time`, `email`, `ipv4`, `ipv6`, `uuid`).
    ///
    /// # Returns
    /// - `true`: If the string matches the specified format.
    /// - `false`: If the string does not match the specified format or if the format is unknown.
    ///
    /// # Supported Formats
    /// - **DATE**: Matches a date in the format `YYYY-MM-DD`.
    /// - **DATE_TIME**: Matches a full date-time string in the format `YYYY-MM-DDTHH:MM:SS` (e.g., ISO 8601).
    /// - **EMAIL**: Matches a valid email address.
    /// - **IPV4**: Matches a valid IPv4 address.
    /// - **IPV6**: Matches a valid IPv6 address.
    /// - **UUID**: Matches a valid UUID string.
    ///
    /// # Behavior
    /// If an unknown format is provided, the function logs a warning and returns `false`
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

    /// Validates a string value against the rules defined in the given schema.
    ///
    /// This function checks whether the provided string value complies with various constraints
    /// specified in the `ObjectSchema`, such as maximum length, minimum length, pattern matching,
    /// and format validation.
    ///
    /// # Parameters
    /// - `value`: The value to validate, expected to be a JSON string (`&Value`).
    /// - `schema`: The `ObjectSchema` containing the constraints to validate against.
    ///
    /// # Returns
    /// - `true`: If the string value meets all the constraints defined in the schema.
    /// - `false`: If the value does not satisfy any of the constraints or is not a string.
    ///
    /// # Validation Rules
    /// - **Maximum Length (`max_length`)**: Ensures the string does not exceed the maximum length.
    /// - **Minimum Length (`min_length`)**: Ensures the string meets the minimum length requirement.
    /// - **Pattern (`pattern`)**: Validates the string against a regular expression.
    /// - **Format (`format`)**: Validates the string format using predefined format rules (e.g., email, date).
    fn validate_string_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(str_val) = value.as_str() {
            if let Some(max) = &schema.max_length {
                println!("Checking if value {} follows max length: {}", str_val, max);
                if *max < str_val.len() as u64 {
                    return false;
                }
            }

            if let Some(min) = &schema.min_length {
                println!("Checking if value {} follows min length: {}", str_val, min);
                if *min > str_val.len() as u64 {
                    return false;
                }
            }

            if let Some(pattern) = &schema.pattern {
                if let Ok(pattern) = regex::Regex::new(pattern) {
                    println!("Checking if value {} follows pattern: {}", str_val, pattern);
                    if !pattern.is_match(str_val) {
                        return false;
                    }
                }
            }

            if let Some(format) = &schema.format {
                println!("Checking if value {} follows format: {}", str_val, format);
                if !Self::validate_string_format(str_val, format) {
                    return false;
                }
            }

            return true;
        }
        false
    }

    /// Validates an integer value against the rules defined in the given schema.
    ///
    /// This function ensures that the provided integer value adheres to constraints
    /// such as maximum, minimum, exclusivity of bounds, multiples, and specific formats
    /// as defined in the `ObjectSchema`.
    ///
    /// # Parameters
    /// - `value`: The value to validate, expected to be a JSON integer (`&Value`).
    /// - `schema`: The `ObjectSchema` containing the constraints to validate against.
    ///
    /// # Returns
    /// - `true`: If the integer value satisfies all the constraints defined in the schema.
    /// - `false`: If the value does not meet any of the constraints or is not an integer.
    ///
    /// # Validation Rules
    /// - **Maximum (`maximum`)**: Ensures the integer does not exceed the maximum value.
    /// - **Minimum (`minimum`)**: Ensures the integer meets the minimum value.
    /// - **Exclusive Maximum (`exclusiveMaximum`)**: Ensures the integer is strictly less than the maximum value.
    /// - **Exclusive Minimum (`exclusiveMinimum`)**: Ensures the integer is strictly greater than the minimum value.
    /// - **Multiple Of (`multipleOf`)**: Ensures the integer is a multiple of the specified value.
    /// - **Format (`format`)**: Validates the integer using predefined format rules.
    fn validate_integer_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(int_val) = value.as_i64() {
            let int_val: i64 = int_val;
            if let Some(max_int) = schema.maximum.as_ref().and_then(|v| v.as_i64()) {
                println!("Checking if value {} follows maximum: {}", int_val, max_int);
                if int_val > max_int {
                    return false;
                }
            }

            if let Some(min_int) = schema.minimum.as_ref().and_then(|v| v.as_i64()) {
                println!("Checking if value {} follows minimum: {}", int_val, min_int);
                if int_val < min_int {
                    return false;
                }
            }

            if let Some(exl_max_int) = schema.exclusive_maximum.as_ref().and_then(|v| v.as_i64()) {
                println!(
                    "Checking if value {} follows exclusive maximum: {}",
                    int_val, exl_max_int
                );
                if int_val >= exl_max_int {
                    return false;
                }
            }

            if let Some(exl_min_int) = schema.exclusive_minimum.as_ref().and_then(|v| v.as_i64()) {
                println!(
                    "Checking if value {} follows exclusive minimum: {}",
                    int_val, exl_min_int
                );
                if int_val <= exl_min_int {
                    return false;
                }
            }

            if let Some(multiple_of) = schema.multiple_of.as_ref().and_then(|v| v.as_i64()) {
                println!(
                    "Checking if value {} is multiple of: {}",
                    int_val, multiple_of
                );
                if int_val % multiple_of != 0 {
                    return false;
                }
            }

            if let Some(format) = &schema.format {
                println!("Checking if value {} follows format: {}", int_val, format);
                if !Self::validate_number_format(value, format) {
                    return false;
                }
            }

            return true;
        }
        false
    }

    /// Validates an array value against the rules defined in the given schema.
    ///
    /// This function ensures that the provided array value complies with various constraints
    /// defined in the `ObjectSchema`, such as the number of items, uniqueness, and item validation
    /// against a schema.
    ///
    /// # Parameters
    /// - `value`: The value to validate, expected to be a JSON array (`&Value`).
    /// - `schema`: The `ObjectSchema` containing the constraints to validate against.
    /// - `spec`: The `Spec` used for resolving schema references and performing additional validation.
    ///
    /// # Returns
    /// - `true`: If the array value adheres to all the constraints defined in the schema.
    /// - `false`: If the value does not satisfy any of the constraints or is not an array.
    ///
    /// # Validation Rules
    /// - **Maximum Items (`max_items`)**: Ensures the array does not contain more than the specified number of items.
    /// - **Minimum Items (`min_items`)**: Ensures the array contains at least the specified number of items.
    /// - **Unique Items (`unique_items`)**: Ensures all elements in the array are unique if this constraint is specified.
    /// - **Item Schema (`items`)**: Validates each element in the array against the defined item schema.
    fn validate_array_rules(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        if let Some(array_val) = value.as_array() {
            if let Some(max_items) = schema.max_items {
                if array_val.len() > max_items as usize {
                    return false;
                }
            }

            if let Some(min_items) = schema.min_items {
                if array_val.len() < min_items as usize {
                    return false;
                }
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
        if let Some(_) = value.as_bool() {
            return true;
        }
        false
    }

    fn validate_null_rules(value: &Value, _schema: &ObjectSchema) -> bool {
        if let Some(_) = value.as_null() {
            return true;
        }
        false
    }

    /// Validates a numeric value against the rules defined in the given schema.
    ///
    /// This function ensures that the provided numeric value complies with constraints
    /// such as maximum, minimum, and exclusive bounds as defined in the `ObjectSchema`.
    ///
    /// # Parameters
    /// - `value`: The value to validate, expected to be a JSON number (`&Value`).
    /// - `schema`: The `ObjectSchema` containing the numeric constraints to validate against.
    ///
    /// # Returns
    /// - `true`: If the numeric value satisfies all the constraints defined in the schema.
    /// - `false`: If the value does not meet any of the constraints or is not a number.
    ///
    /// # Validation Rules
    /// - **Maximum (`maximum`)**: Ensures the number does not exceed the maximum value.
    /// - **Minimum (`minimum`)**: Ensures the number is not less than the minimum value.
    /// - **Exclusive Maximum (`exclusiveMaximum`)**: Ensures the number is strictly less than the exclusive maximum value.
    /// - **Exclusive Minimum (`exclusiveMinimum`)**: Ensures the number is strictly greater than the exclusive minimum value.
    fn validate_number_rules(value: &Value, schema: &ObjectSchema) -> bool {
        if let Some(number_val) = value.as_f64() {
            if let Some(max) = schema.maximum.as_ref().and_then(|v| v.as_f64()) {
                println!("Checking if value {} follows maximum: {}", number_val, max);
                if number_val > max {
                    return false;
                }
            }

            if let Some(min) = schema.minimum.as_ref().and_then(|v| v.as_f64()) {
                println!("Checking if value {} follows minimum: {}", number_val, min);
                if number_val < min {
                    return false;
                }
            }

            if let Some(exl_max) = schema.exclusive_maximum.as_ref().and_then(|v| v.as_f64()) {
                println!(
                    "Checking if value {} follows exclusive maximum: {}",
                    number_val, exl_max
                );
                if number_val >= exl_max {
                    return false;
                }
            }

            if let Some(exl_min) = schema.exclusive_minimum.as_ref().and_then(|v| v.as_f64()) {
                println!(
                    "Checking if value {} follows exclusive minimum: {}",
                    number_val, exl_min
                );
                if number_val <= exl_min {
                    return false;
                }
            }

            return true;
        }
        false
    }

    /// Validates an object value against the rules defined in the given schema.
    ///
    /// This function ensures that the provided object adheres to constraints such as
    /// the presence of required fields and the validation of properties based on their schemas.
    ///
    /// # Parameters
    /// - `value`: The value to validate, expected to be a JSON object (`&Value`).
    /// - `schema`: The `ObjectSchema` containing the object constraints to validate against.
    /// - `spec`: The `Spec` used for resolving schema references and performing additional validation.
    ///
    /// # Returns
    /// - `true`: If the object value satisfies all the constraints defined in the schema.
    /// - `false`: If the value does not meet any constraints or is not an object.
    ///
    /// # Validation Rules
    /// - **Required Fields**: Ensures that all fields specified in the schema's `required` list are present in the object.
    /// - **Properties**: Validates each property of the object against its corresponding schema. If a schema reference is provided, it is resolved before validation.
    fn validate_object_rules(value: &Value, schema: &ObjectSchema, spec: &Spec) -> bool {
        if let Some(object_val) = value.as_object() {
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
                println!("Validation object property {} ", field);
                if let Ok(resolved_schema) = field_schema.resolve(&spec) {
                    if let Some(object_field_val) = object_val.get(field) {
                        if !Self::validate_with_schema(object_field_val, &resolved_schema, spec) {
                            return false;
                        }
                    }
                }
            }

            return true;
        }
        false
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
                SchemaTypeSet::Multiple(type_vals) => type_vals
                    .iter()
                    .any(|type_val| Self::validate_for_type(value, type_val, schema, spec)),
            };

        // Check if for oneOf, anyOf, allOf, and not
        // NOTE: oas3 has not implemented 'not' yet
        } else if !schema.one_of.is_empty()
            || !schema.all_of.is_empty()
            || !schema.any_of.is_empty()
        /* !schema.not.is_empty() */
        {
            let mut valid_count = 0usize;
            if !schema.all_of.is_empty() {
                let goal_matches = schema.all_of.len();
                for all_of_schema in &schema.all_of {
                    if let Ok(resolved_schema) = all_of_schema.resolve(&spec) {
                        if Self::validate_with_schema(value, &resolved_schema, spec) {
                            valid_count += 1;
                        }
                    }
                }

                // all matches
                if valid_count == goal_matches {
                    return true;
                }
            } else if !schema.any_of.is_empty() {
                for any_of_schema in &schema.any_of {
                    if let Ok(resolved_schema) = any_of_schema.resolve(&spec) {
                        if Self::validate_with_schema(value, &resolved_schema, spec) {
                            valid_count += 1;
                        }
                    }
                }

                // any matches
                if valid_count > 0 {
                    return true;
                }
            } else if !schema.one_of.is_empty() {
                for any_of_schema in &schema.one_of {
                    if let Ok(resolved_schema) = any_of_schema.resolve(&spec) {
                        if Self::validate_with_schema(value, &resolved_schema, spec) {
                            valid_count += 1;
                        }
                    }
                }

                // one and only one match
                if valid_count == 1 {
                    return true;
                }
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

    fn match_path_segments(
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
                if let (Some(start), Some(end)) = (spec_segment.find(Self::PATH_PARAM_LEFT), spec_segment.find(Self::PATH_PARAM_RIGHT)) {
                    let spec_segment_param_name = &spec_segment[start + 1..end];
                    if let Some(spec_segment_param_value) =
                        self.get_param_value(spec_segment_param_name, path_method_item_params)
                    {
                        if let Some(resolved_schema) = spec_segment_param_value.schema {
                            if let Ok(schema) = resolved_schema.resolve(&self.specification) {
                                if Self::validate_with_schema(&json!(target_segment), &schema, &self.specification) {
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

    fn validate_parameters(
        headers: &HashMap<String, String>,
        parameters: &Vec<&ObjectOrReference<Parameter>>,
        spec: &Spec,
    ) -> bool {
        for parameter in parameters {
            if let Ok(resolved_param) = parameter.resolve(&spec) {
                if let Some((_, header_value)) = headers
                    .iter()
                    .find(|(header_name, _)| header_name.as_str() == resolved_param.name)
                {
                    if let Some(_) = resolved_param.content {
                        // TODO - can content be defined in a parameter?
                        todo!("validating 'content' field in parameters not implemented yet")
                    } else if let Some(schema) = resolved_param.schema {
                        if let Ok(schema) = schema.resolve(&spec) {
                            if !Self::validate_with_schema(
                                &Value::String(header_value.clone()),
                                &schema,
                                &spec,
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

                /* filter params to Header params only and validate */
                if let Some(headers) = headers {
                    let relevant_parameters = parameters
                        .iter()
                        .filter(|param| {
                            param
                                .resolve(&self.specification)
                                .is_ok_and(|param| param.location == ParameterIn::Header)
                        })
                        .collect::<Vec<&ObjectOrReference<Parameter>>>();
                    if !Self::validate_parameters(
                        headers,
                        &relevant_parameters,
                        &self.specification,
                    ) {
                        return Err(());
                    }
                }

                /* filter params to Query params only and validate */
                if let Some(query_params) = query_params {
                    let relevant_parameters = parameters
                        .iter()
                        .filter(|param| {
                            param
                                .resolve(&self.specification)
                                .is_ok_and(|param| param.location == ParameterIn::Query)
                        })
                        .collect::<Vec<&ObjectOrReference<Parameter>>>();
                    if !Self::validate_parameters(
                        query_params,
                        &relevant_parameters,
                        &self.specification,
                    ) {
                        return Err(());
                    }
                }
            }

            return Ok(())
        }

        Err(())
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
            Some(&test_request_headers),
            None,
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
