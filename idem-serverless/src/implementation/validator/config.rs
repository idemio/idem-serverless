use serde::Deserialize;

#[derive(Deserialize)]
pub struct ValidatorHandlerConfig {
    pub enable: bool,
    pub validate_request: bool,
    pub validate_response: bool,
    pub openapi_specification: String
}

impl Default for ValidatorHandlerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            validate_request: true,
            validate_response: false,
            openapi_specification: "openapi.json".to_string()
        }
    }
}