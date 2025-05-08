use crate::{try_cast_to_type, validate_with_schema};
use oas3::Spec;
use oas3::spec::{ObjectOrReference, ObjectSchema, Operation, Parameter, PathItem, SchemaTypeSet};
use serde_json::Value;
use std::collections::BTreeMap;
use std::str::FromStr;

pub struct OpenApiNodeFinder;

#[derive(Debug, Clone)]
pub struct JsonPath(pub Vec<String>);

impl JsonPath {
    pub fn new() -> Self {
        JsonPath(Vec::new())
    }

    pub fn add_segment(&mut self, segment: String) -> &mut Self {
        if segment.contains("/") {
            let segment = segment.replace("/", "~1");
            self.0.push(segment);
        } else {
            self.0.push(segment);
        }
        self
    }

    pub fn append_path(&mut self, path: JsonPath) -> &mut Self {
        let mut path = path;
        self.0.append(&mut path.0);
        self
    }

    pub fn format_path(&self) -> String {
        self.0.join("/")
    }
}

impl OpenApiNodeFinder {
    const PATHS_KEY: &'static str = "paths";
    const PATH_SPLIT: char = '/';
    const PATH_PARAM_LEFT: char = '{';
    const PATH_PARAM_RIGHT: char = '}';

    /// Performs a search for a specific path and HTTP method in the OpenAPI specification.
    ///
    /// This function searches through all paths in the OpenAPI spec to find a match for the given
    /// path and method, with the option of taking path parameters into account during matching.
    /// For example, an OpenAPI path `/users/{id}` would match an input path like `/users/123`.
    ///
    /// The function checks each path and method combination, and when it finds a match, it constructs
    /// a JsonPath that points to the exact location of the operation in the specification.
    ///
    /// # Arguments
    ///
    /// * `path_to_match` - The concrete API path to search for (e.g., "/users/123")
    /// * `method_to_match` - The HTTP method to match (e.g., "GET", "POST", "PUT")
    /// * `spec` - The complete OpenAPI specification, needed for parameter resolution
    /// * `path_param` - Indicate if you want to do a detailed search because the
    ///                  path provided contains path parameters.
    ///
    /// # Returns
    ///
    /// If a match is found, returns `Some((operation, json_path))` where:
    /// * `operation` is a reference to the matching Operation object
    /// * `json_path` is the JsonPath pointing to the operation in the spec
    ///
    /// Returns `None` if no matching path and method combination is found.
    ///
    /// # Examples
    ///
    /// ```
    /// use oas3::Spec;
    /// use std::collections::BTreeMap;
    /// use std::fs;
    /// use idem_openapi::node_finder::OpenApiNodeFinder;
    ///
    ///
    /// let json_string = fs::read_to_string("test/openapi").unwrap();
    /// let spec = oas3::from_json(json_string).unwrap();
    ///
    ///
    /// let result = OpenApiNodeFinder::find_matching_operation("/pet/findById/42", "GET", &spec, true);
    /// assert!(result.is_some());
    ///
    /// let result = OpenApiNodeFinder::find_matching_operation("/pet/findPById/42", "GET", &spec, false);
    /// assert!(result.is_none());
    /// ```
    /// Note: Path parameters were a mistake :^)
    ///
    pub fn find_matching_operation<'a>(
        path_to_match: &'a str,
        method_to_match: &'a str,
        spec: &'a Spec,
        path_param: bool,
    ) -> Option<(&'a Operation, JsonPath)> {
        let spec_paths = match &spec.paths {
            Some(paths) => paths,
            None => return None,
        };

        if path_param {
            Self::detailed_path_search(path_to_match, method_to_match, spec_paths, spec)
        } else {
            if let Some(op) = spec.operation(
                &http::method::Method::from_str(method_to_match).unwrap(),
                path_to_match,
            ) {
                let mut path = JsonPath::new();
                path.add_segment(Self::PATHS_KEY.to_string())
                    .add_segment(path_to_match.to_string())
                    .add_segment(method_to_match.to_lowercase().to_string());
                return Some((op, path));
            }
            None
        }
    }

    fn detailed_path_search<'a>(
        path_to_match: &'a str,
        method_to_match: &'a str,
        paths: &'a BTreeMap<String, PathItem>,
        spec: &'a Spec,
    ) -> Option<(&'a Operation, JsonPath)> {

        // Find the matching method
        for (spec_path, path_item) in paths.iter() {
            if let Some((_, op)) = path_item
                .methods()
                .into_iter()
                .find(|(method, _)| method.as_str() == method_to_match)
            {
                // Perform our check to see if this matches
                let path_method_item_params = &op.parameters;
                if Self::match_openapi_endpoint_path_segments(
                    path_to_match,
                    spec_path,
                    path_method_item_params,
                    spec,
                ) {
                    let mut path = JsonPath::new();
                    path.add_segment(Self::PATHS_KEY.to_string())
                        .add_segment(spec_path.to_string())
                        .add_segment(method_to_match.to_lowercase().to_string());
                    return Some((op, path));
                }
            }
        }
        None
    }

    fn match_openapi_endpoint_path_segments(
        target_path: &str,
        spec_path: &str,
        path_method_item_params: &Vec<ObjectOrReference<Parameter>>,
        spec: &Spec,
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
                    if let Some(param_name) =
                        Self::extract_openapi_path_parameter_name(spec_segment)
                    {
                        match Self::path_parameter_value_matches_type(
                            param_name,
                            target_segment,
                            path_method_item_params,
                            spec,
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
    fn extract_openapi_path_parameter_name(segment: &str) -> Option<&str> {
        segment.find(Self::PATH_PARAM_LEFT).and_then(|start| {
            segment
                .find(Self::PATH_PARAM_RIGHT)
                .map(|end| &segment[start + 1..end])
        })
    }

    fn get_path_parameter_definition(
        param_name: &str,
        endpoint_params: &Vec<ObjectOrReference<Parameter>>,
        spec: &Spec,
    ) -> Option<Parameter> {

        // look through each parameter for the operation, if the 'name' field value
        // matches the provided 'param_name' then return that.
        // returns None if there is no matching parameter schema for the operation.
        endpoint_params.iter().find_map(|param| {
            param.resolve(&spec).ok().and_then(|param| {
                if param_name == param.name.as_str() {
                    Some(param.clone())
                } else {
                    None
                }
            })
        })
    }

    fn path_parameter_value_matches_type(
        param_name: &str,
        target_segment: &str,
        path_method_item_params: &Vec<ObjectOrReference<Parameter>>,
        spec: &Spec,
    ) -> Result<(), ()> {
        if let Some(param) =
            Self::get_path_parameter_definition(param_name, path_method_item_params, spec)
        {
            if let Some(resolved_schema) =
                param.schema.and_then(|schema| schema.resolve(&spec).ok())
            {
                if let Ok(value) =
                    Self::try_cast_path_param_to_schema_type(target_segment, &resolved_schema)
                {
                    let value = &value;
                    let res = validate_with_schema(value, &resolved_schema);
                    return res;
                }
            }
        }
        Ok(())
    }

    fn try_cast_path_param_to_schema_type(
        target_segment: &str,
        schema: &ObjectSchema,
    ) -> Result<Value, ()> {
        let param_type = schema.schema_type.as_ref().unwrap();
        match param_type {
            SchemaTypeSet::Single(single) => try_cast_to_type(target_segment, &single),
            SchemaTypeSet::Multiple(multi) => {
                for m_type in multi {
                    let res = try_cast_to_type(target_segment, &m_type);
                    if res.is_ok() {
                        return res;
                    }
                }
                Err(())
            }
        }
    }
}
