use serde::Deserialize;
use crate::implementation::jwt::jwk_provider::JwkProviders;

#[derive(Deserialize)]
pub struct JwtValidationHandlerConfig {
    pub enabled: bool,
    pub jwk_provider: JwkProviders,
    pub scope_verification: bool,
    pub specification_name: String,
    pub ignore_jwt_expiration: bool,
    pub audience: String,

}

impl Default for JwtValidationHandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwk_provider: JwkProviders::default(),
            scope_verification: false,
            ignore_jwt_expiration: false,
            specification_name: "openapi.yaml".to_string(),
            audience: "https://issuer.example.com".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::implementation::jwt::config::JwtValidationHandlerConfig;
    use crate::implementation::jwt::jwk_provider::JwkProvider;

    #[test]
    fn load_jwk_file_test() {
        let file = r#"
        {
            "enabled": true,
            "scope_verification": false,
            "ignore_jwt_expiration": false,
            "specification_name": "openapi.yaml",
            "jwk_provider": {
                "LocalJwkProvider": {
                    "file_name": "jwks.json",
                    "file_path": "./config"
                 }
            },
            "audience": "https://issuer.example.com"
        }
        "#;
        let jwt_config: JwtValidationHandlerConfig = serde_json::from_str(file).unwrap();
        assert!(jwt_config.enabled);
        let jwk_set = jwt_config.jwk_provider.jwk().unwrap();
        assert!(jwk_set.keys.iter().any(|jwk| jwk.clone().common.key_id.unwrap() == "DDbt045YVtnjCkzHUv-rFN4wPfGD3Upk9_da_yweZ1c"));
    }
}



