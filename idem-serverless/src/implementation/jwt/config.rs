use serde::Deserialize;
use crate::implementation::jwt::jwk_provider::JwkProviders;

#[derive(Deserialize)]
pub struct JwtValidationHandlerConfig {
    pub enabled: bool,
    pub jwk_provider: JwkProviders,
    pub audience: String,

}

impl Default for JwtValidationHandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwk_provider: JwkProviders::default(),
            audience: "https://issuer.example.com".to_string(),
        }
    }
}



