use std::convert::Infallible;
use crate::ROOT_CONFIG_PATH;
use crate::handler::LambdaExchange;
use async_trait::async_trait;
use idemio::config::Config;
use idemio::exchange::Exchange;
use idemio::handler::Handler;
use idemio::status::{ExchangeState, HandlerStatus};
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use oasert::validator::{OpenApiPayloadValidator};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
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

pub trait JwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()>;
}

#[derive(Deserialize, Default, Debug)]
pub struct LocalJwkProvider {
    file_name: String,
    file_path: String,
}

impl JwkProvider for LocalJwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()> {
//        let file = get_file(&format!("{}/{}", self.file_path, self.file_name)).unwrap();
//        serde_json::from_str(&file).or(Err(()))
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub enum JwkProviders {
    RemoteJwkProvider(RemoteJwkProvider),
    LocalJwkProvider(LocalJwkProvider),
}

#[derive(Deserialize, Default, Debug)]
pub struct RemoteJwkProvider {
    jwk_server_url: String,
    jwk_server_path: String,
}

impl JwkProvider for RemoteJwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()> {
        todo!()
    }
}

impl Default for JwkProviders {
    fn default() -> Self {
        Self::LocalJwkProvider(LocalJwkProvider {
            file_name: String::from("jwks.json"),
            file_path: String::from("./config"),
        })
    }
}

impl JwkProvider for JwkProviders {
    fn jwk(&self) -> Result<JwkSet, ()> {
        match self {
            JwkProviders::LocalJwkProvider(local) => local.jwk(),

            JwkProviders::RemoteJwkProvider(remote) => remote.jwk(),
        }
    }
}

//#[derive(ConfigurableHandler)]
pub struct JwtValidationHandler {
    pub(crate) config: Config<JwtValidationHandlerConfig>,
}

impl JwtValidationHandler {
    fn fetch_jwk(&self) -> Result<JwkSet, ()> {
        self.config.get().jwk_provider.jwk()
    }

    fn validate_scope(spec: Value, request_path: &str, method: &str, claims: &Value) -> Result<(), ()> {
        let token_scopes = match claims.get("scope") {
            None => return Err(()),
            Some(scope) => {
                if let Some(scope) = scope.as_str() {
                    scope.split(' ').map(String::from).collect::<Vec<String>>()
                } else {
                    return Err(());
                }
            }
        };


        let validator = match OpenApiPayloadValidator::new(spec) {
            Ok(x) => x,
            Err(_) => return Err(()),
        };
        let operation = match validator.traverser().get_operation_from_path_and_method(request_path, method) {
            Ok(x) => x,
            Err(_) => return Err(()),
        };
        if let Err(_) = validator.validate_request_scopes(&operation, &token_scopes) {
            return Err(());
        }
        Ok(())
    }

    fn validate_aud(&self, claims: &Value) -> Result<(), ()> {
        Ok(())
    }

    fn validate_iss(&self, claims: &Value) -> Result<(), ()> {
        Ok(())
    }

    fn validate_exp(&self, claims: &Value) -> Result<(), ()> {
        Ok(())
    }
}

#[async_trait]
impl Handler<Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>> for JwtValidationHandler {

    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) -> Result<HandlerStatus, Infallible> {
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(ExchangeState::DISABLED));
        }

        let request = match exchange.input().await {
            Ok(req) => req,
            Err(_) => {
                return Ok(
                    HandlerStatus::new(ExchangeState::SERVER_ERROR).message("Unable to get request")
                );
            }
        };

        if let Some((_, auth_header_value)) = &request
            .headers
            .iter()
            .find(|(header_key, _)| header_key.to_string().to_lowercase() == "authorization")
        {
            let auth_header_parts = auth_header_value
                .to_str()
                .unwrap()
                .split(' ')
                .collect::<Vec<&str>>();

            if auth_header_parts.len() != 2 || !(auth_header_parts[0].to_lowercase() == "bearer") {
                return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                    .message("Missing client bearer token header"));
            }

            let token = auth_header_parts[1];

            let jwk_set = match self.fetch_jwk() {
                Ok(jwk_set) => jwk_set,
                Err(_) => {
                    return Ok(
                        HandlerStatus::new(ExchangeState::SERVER_ERROR).message("Unable to fetch JWKs")
                    );
                }
            };

            let header = match decode_header(token) {
                Ok(jwt_header) => jwt_header,
                Err(_) => {
                    return Ok(
                        HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Malformed JWT header")
                    );
                }
            };

            let kid = match header.kid {
                Some(kid) => kid,
                None => {
                    return Ok(
                        HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("JWT is missing kid")
                    );
                }
            };

            let matching_jwk = match jwk_set.find(&kid) {
                Some(matching_jwk) => matching_jwk,
                None => {
                    return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                        .message("No matching JWK for kid"));
                }
            };
            let decoding_key = match &matching_jwk.algorithm {
                AlgorithmParameters::RSA(rsa_params) => {
                    match DecodingKey::from_rsa_components(&rsa_params.n, &rsa_params.e) {
                        Ok(decoding_key) => decoding_key,
                        Err(_) => {
                            return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                                .message("Malformed RSA key"));
                        }
                    }
                }
                _ => {
                    return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                        .message("Unsupported JWT algorithm"));
                }
            };

            let validation = Validation::new(Algorithm::RS256);
            let token_data = match decode::<Value>(token, &decoding_key, &validation) {
                Ok(token_data) => token_data,
                Err(_) => {
                    return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Invalid JWT"));
                }
            };

            let claims = token_data.claims;
            let (request_path, method) = match (&request.path, &request.http_method) {
                (None, _) => {
                    return Ok(
                        HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Missing request path")
                    );
                }
                (Some(path), method) => (path, method),
            };

            if self.config.get().scope_verification {
                let spec =
                    match std::fs::read_to_string(&format!("{}/{}", ROOT_CONFIG_PATH, &self.config.get().specification_name)) {
                        Ok(file) => file,
                        Err(_) => todo!(),
                    };
                let spec = match serde_json::from_str(&spec) {
                    Ok(x) => x,
                    Err(_) => todo!(),
                };
                if let Err(_) = Self::validate_scope(spec, &request_path, &method.to_string(), &claims) {
                    return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                        .message("Invalid scope for token"));
                }
            }

            if let Err(_) = self.validate_aud(&claims) {
                return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR)
                    .message("Invalid audience for token"));
            }

            if let Err(_) = self.validate_iss(&claims) {
                return Ok(
                    HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Invalid issuer for token")
                );
            }

            if let Err(_) = self.validate_exp(&claims) {
                return Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Expired token"));
            }

            Ok(HandlerStatus::new(ExchangeState::OK))
        } else {
            Ok(HandlerStatus::new(ExchangeState::CLIENT_ERROR).message("Missing JWT"))
        }
    }

    fn name(&self) -> &str {
        "JwtValidationHandler"
    }
}

#[cfg(test)]
mod test {
    use crate::handler::LambdaExchange;
    use crate::handler::jwt::{JwkProvider, JwkProviders, JwtValidationHandler, JwtValidationHandlerConfig};
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequest;
    use lambda_http::http::HeaderValue;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::fs::File;
    use idemio::config::{Config, DefaultConfigProvider};
    use idemio::exchange::Exchange;
    use idemio::handler::Handler;
    use idemio::status::ExchangeState;
    use serde_json::{json, Value};

    fn b64_decode(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(BASE64_URL_SAFE_NO_PAD.decode(s)?)
    }

    fn rsa_private_key_from_jwk(jwk: &serde_json::Value) -> Result<RsaPrivateKey, Box<dyn Error>> {
        let n = rsa::BigUint::from_bytes_be(&b64_decode(jwk["n"].as_str().unwrap())?);
        let e = rsa::BigUint::from_bytes_be(&b64_decode(jwk["e"].as_str().unwrap())?);
        let d = rsa::BigUint::from_bytes_be(&b64_decode(jwk["d"].as_str().unwrap())?);
        let p = rsa::BigUint::from_bytes_be(&b64_decode(jwk["p"].as_str().unwrap())?);
        let q = rsa::BigUint::from_bytes_be(&b64_decode(jwk["q"].as_str().unwrap())?);
        Ok(RsaPrivateKey::from_components(n, e, d, vec![p, q]).unwrap())
    }

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    fn get_test_key_gen() -> String {
        let test_file = File::open("./test_resources/jwt/public_private_keypair.json");
        let jwk: serde_json::Value = serde_json::from_reader(test_file.unwrap()).unwrap();
        let private_key = rsa_private_key_from_jwk(&jwk).unwrap();
        let der = private_key.to_pkcs1_der().unwrap().as_bytes().to_vec();
        let encoding_key = EncodingKey::from_rsa_der(&der);
        let claims = Claims {
            sub: "user123".to_string(),
            exp: 2000000000,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = jwk
            .get("kid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        encode(&header, &claims, &encoding_key).unwrap()
    }

    #[test]
    fn create_jwt_test() {
        let token = get_test_key_gen();
        println!("{}", token);
        assert!(true)
    }

    #[test]
    fn test_load_config() {
        let config_string = r#"{
          "enabled": true,
          "jwk_provider": {
            "LocalJwkProvider": {
              "file_name": "jwks.json",
              "file_path": "./config"
            }
          },
          "scope_verification": true,
          "ignore_jwt_expiration": true,
          "specification_name": "openapi.json",
          "audience": ""
        }"#;
        let config = serde_json::from_str::<JwtValidationHandlerConfig>(config_string);
        println!("config: {:?}", config);
        assert!(config.is_ok());

    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_valid_jwt_validator_handler() {
        // generate a valid token from a test pub private key set
        let token = get_test_key_gen();
        let complete_token_header = format!("{} {}", "Bearer", token);

        // create a request containing our valid jwt and execute the handler
        let mut test_request = ApiGatewayProxyRequest::default();
        test_request.path = Some("/test".to_string());
        test_request.headers.insert(
            "Authorization",
            HeaderValue::from_str(&complete_token_header).unwrap(),
        );
        let mut test_exchange: LambdaExchange = Exchange::new();
        test_exchange.set_input(test_request);
//        let jwt_validation_handler =
//            JwtValidationHandler::init_handler(Config::new(DefaultConfigProvider).unwrap());

        let jwt_validation_handler = JwtValidationHandler {
            config: Config::new(DefaultConfigProvider).unwrap()
        };


        // make sure the result is OK
        let result = jwt_validation_handler
            .exec(&mut test_exchange)
            .await
            .unwrap();
        let result_code = result.code();
        if result_code.any_flags(ExchangeState::OK) {
            assert!(
                true,
                "Handler returned an OK status meaning validation passed"
            )
        } else {
            assert!(
                false,
                "Handler returned something other than OK status meaning validation did not pass"
            )
        }
    }


    #[tokio::test(flavor = "current_thread")]
    async fn test_invalid_jwt_validator_handler() {
        // An invalid/malformed JWT token
        let invalid_token = "Bearer 389475983475893745invalid_jwt4789234789";

        // Create an exchange containing the header with our invalid token.
        let mut test_request = ApiGatewayProxyRequest::default();
        test_request.path = Some("/test".to_string());
        test_request.headers.insert(
            "Authorization",
            HeaderValue::from_str(&invalid_token).unwrap(),
        );
        let mut test_exchange: LambdaExchange = Exchange::new();
        test_exchange.set_input(test_request);

        // execute the validation and get the result
//        let jwt_validation_handler =
//            JwtValidationHandler::init_handler(Config::new(DefaultConfigProvider).unwrap());
        let jwt_validation_handler = JwtValidationHandler {
            config: Config::new(DefaultConfigProvider).unwrap()
        };
        let result = jwt_validation_handler
            .exec(&mut test_exchange)
            .await
            .unwrap();

        assert!(result.code().any_flags(ExchangeState::CLIENT_ERROR));

        // make sure we returned the client error code with the Malformed 'JWT header message'
//        let result_code = result.code();
//        let result_message = result
//        if result_code.any_flags(ExchangeState::CLIENT_ERROR) && result_message == "Malformed JWT header" {
//            assert!(true)
//        } else {
//            assert!(false)
//        }
    }

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
        assert!(
            jwk_set
                .keys
                .iter()
                .any(|jwk| jwk.clone().common.key_id.unwrap()
                    == "DDbt045YVtnjCkzHUv-rFN4wPfGD3Upk9_da_yweZ1c")
        );
    }

    fn create_test_spec() -> Value {
        json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/users": {
                    "get": {
                        "security": [
                            {
                                "oauth2": ["read:users"]
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    },
                    "post": {
                        "security": [
                            {
                                "oauth2": ["write:users"]
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    }
                },
                "/admin": {
                    "get": {
                        "security": [
                            {
                                "oauth2": ["admin:read", "admin:users"]
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    }
                },
                "/public": {
                    "get": {
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    }
                },
                "/alternate-auth": {
                    "get": {
                        "security": [
                            {
                                "oauth2": ["read:resource"]
                            },
                            {
                                "oauth2": ["admin:all"]
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "OK"
                            }
                        }
                    }
                }
            },
            "components": {
                "securitySchemes": {
                    "oauth2": {
                        "type": "oauth2",
                        "flows": {
                            "implicit": {
                                "authorizationUrl": "https://example.com/oauth/authorize",
                                "scopes": {
                                    "read:users": "Read user data",
                                    "write:users": "Write user data",
                                    "admin:read": "Admin read access",
                                    "admin:users": "Admin user access",
                                    "read:resource": "Read resource",
                                    "admin:all": "Admin all access"
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn test_validate_scope_with_valid_single_scope() {
        let spec = create_test_spec();
        let claims = json!({"scope": "read:users"});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_ok(), "Token with correct scope should be valid");
    }

    #[test]
    fn test_validate_scope_with_missing_required_scope() {
        let spec = create_test_spec();
        let claims = json!({"scope": "write:users"});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_err(), "Token with wrong scope should be invalid");
    }

    #[test]
    fn test_validate_scope_with_multiple_scopes_including_required() {
        let spec = create_test_spec();
        let claims = json!({"scope": "read:users write:users admin:read"});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_ok(), "Token with multiple scopes including required one should be valid");
    }

    #[test]
    fn test_validate_scope_with_different_http_method() {
        let spec = create_test_spec();
        let claims = json!({"scope": "write:users"});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "POST", &claims);

        assert!(result.is_ok(), "Token with correct scope for POST should be valid");
    }

    #[test]
    fn test_validate_scope_with_endpoint_requiring_multiple_scopes() {
        let spec = create_test_spec();

        // Test with all required scopes
        let claims_complete = json!({"scope": "admin:read admin:users"});
        let result_complete = JwtValidationHandler::validate_scope(spec.clone(), "/admin", "GET", &claims_complete);
        assert!(result_complete.is_ok(), "Token with all required scopes should be valid");

        // Test with only one of the required scopes
        let claims_partial = json!({"scope": "admin:read"});
        let result_partial = JwtValidationHandler::validate_scope(spec, "/admin", "GET", &claims_partial);
        assert!(result_partial.is_err(), "Token with only some required scopes should be invalid");
    }

    #[test]
    fn test_validate_scope_with_missing_scope_claim() {
        let spec = create_test_spec();
        let claims = json!({});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_err(), "Token without scope claim should be invalid");
    }

    #[test]
    fn test_validate_scope_with_non_string_scope() {
        let spec = create_test_spec();
        let claims = json!({"scope": 123});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_err(), "Token with non-string scope should be invalid");
    }

    #[test]
    fn test_validate_scope_with_public_endpoint() {
        let spec = create_test_spec();
        let claims = json!({"scope": ""});

        let result = JwtValidationHandler::validate_scope(spec, "/public", "GET", &claims);

        assert!(result.is_ok(), "Public endpoint should not require scopes");
    }

    #[test]
    fn test_validate_scope_with_nonexistent_path() {
        let spec = create_test_spec();
        let claims = json!({"scope": "read:users"});

        // This test will verify the error handling in the validator.traverser().get_operation call
        let result = JwtValidationHandler::validate_scope(spec, "/nonexistent", "GET", &claims);

        // The exact result depends on the implementation of the error handling in get_operation,
        // but we expect an error since the path doesn't exist
        assert!(result.is_err(), "Nonexistent path should result in an error");
    }

    #[test]
    fn test_validate_scope_with_invalid_method() {
        let spec = create_test_spec();
        let claims = json!({"scope": "read:users"});

        // This test will verify the error handling in the validator.traverser().get_operation call
        let result = JwtValidationHandler::validate_scope(spec, "/users", "INVALID_METHOD", &claims);

        // The exact result depends on the implementation of the error handling in get_operation,
        // but we expect an error since the method doesn't exist
        assert!(result.is_err(), "Invalid method should result in an error");
    }

    #[test]
    fn test_validate_scope_with_alternate_security_requirements() {
        let spec = create_test_spec();

        // Test with first security requirement
        let claims_first = json!({"scope": "read:resource"});
        let result_first = JwtValidationHandler::validate_scope(spec.clone(), "/alternate-auth", "GET", &claims_first);
        assert!(result_first.is_ok(), "Token with first alternate scope should be valid");

        // Test with second security requirement
        let claims_second = json!({"scope": "admin:all"});
        let result_second = JwtValidationHandler::validate_scope(spec, "/alternate-auth", "GET", &claims_second);
        assert!(result_second.is_ok(), "Token with second alternate scope should be valid");
    }

    #[test]
    fn test_validate_scope_with_empty_scope() {
        let spec = create_test_spec();
        let claims = json!({"scope": ""});

        let result = JwtValidationHandler::validate_scope(spec, "/users", "GET", &claims);

        assert!(result.is_err(), "Empty scope should be invalid for protected endpoint");
    }

    #[test]
    fn test_validate_scope_with_malformed_spec() {
        let malformed_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Malformed API",
                "version": "1.0.0"
            }
            // Missing required fields
        });

        let claims = json!({"scope": "read:users"});

        // This test will verify the error handling in the OpenApiPayloadValidator::new call
        let result = JwtValidationHandler::validate_scope(malformed_spec, "/users", "GET", &claims);

        // The exact result depends on the implementation of the error handling in OpenApiPayloadValidator::new,
        // but we expect an error since the spec is malformed
        assert!(result.is_err(), "Malformed spec should result in an error");
    }

}
