use serde::Deserialize;
use idem_handler_config::config_cache::get_file;
use async_trait::async_trait;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use idem_handler_config::config::Config;
use idem_handler_macro::ConfigurableHandler;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_http::Context;
use serde_json::Value;
use crate::handler::LambdaExchange;

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



pub trait JwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()>;
}

#[derive(Deserialize, Default)]
pub struct LocalJwkProvider {
    file_name: String,
    file_path: String,
}

impl JwkProvider for LocalJwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()> {
        let file = get_file(&format!("{}/{}", self.file_path, self.file_name)).unwrap();
        serde_json::from_str(&file).or(Err(()))
    }
}

#[derive(Deserialize, Default)]
pub struct RemoteJwkProvider {
    jwk_server_url: String,
    jwk_server_path: String,
}

impl JwkProvider for RemoteJwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()> {
        todo!()
    }
}

#[derive(Deserialize)]
pub enum JwkProviders {
    RemoteJwkProvider(RemoteJwkProvider),
    LocalJwkProvider(LocalJwkProvider),
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



#[derive(ConfigurableHandler)]
pub struct JwtValidationHandler {
    config: Config<JwtValidationHandlerConfig>,
}

impl JwtValidationHandler {
    fn fetch_jwk(&self) -> Result<JwkSet, ()> {
        self.config.get().jwk_provider.jwk()
    }

    fn validate_scope(&self, request_path: &str, method: &str, claims: &Value) -> Result<(), ()> {
        todo!()
        //        let spec_validator =
        //            OpenApiValidator::from_file(&format!("{}/{}", ROOT_CONFIG_PATH, "openapi.json"));
        //        let schemas = spec_validator.get_security_scopes(request_path, method);
        //        let token_scopes = match claims.get("scope") {
        //            None => return Err(()),
        //            Some(scope) => {
        //                if let Some(scope) = scope.as_str() {
        //                    scope.split(' ').collect::<Vec<&str>>()
        //                } else {
        //                    return Err(());
        //                }
        //            }
        //        };
        //        if let Some(schemas) = schemas {
        //            for (_, scopes) in schemas {
        //                let potential_matched_scope = scopes.iter().find(|scope| {
        //                    token_scopes.iter().any(|token_scope| scope == token_scope)
        //                });
        //
        //                if potential_matched_scope.is_some() {
        //                    return Ok(());
        //                }
        //            }
        //        }
        //        Err(())
    }

    fn validate_aud(&self, claims: &Value) -> Result<(), ()> {
        todo!()
    }

    fn validate_iss(&self, claims: &Value) -> Result<(), ()> {
        todo!()
    }

    fn validate_exp(&self, claims: &Value) -> Result<(), ()> {
        todo!()
    }
}

#[async_trait]
impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for JwtValidationHandler {
    async fn exec(
        &self,
        exchange: &mut LambdaExchange,
    ) ->  Result<HandlerStatus, HandlerExecutionError>
    {
        if !self.config.get().enabled {
            return Ok(HandlerStatus::new(Code::DISABLED));
        }

        let request = match exchange.input() {
            Ok(req) => req,
            Err(_) => {
                return Ok(
                    HandlerStatus::new(Code::SERVER_ERROR).set_message("Unable to get request")
                )
            }
        };

        if let Some((_, auth_header_value)) = &request
            .headers
            .iter()
            .find(|(header_key, _)| header_key.to_string().to_lowercase() == "Authorization")
        {
            let auth_header_parts = auth_header_value
                .to_str()
                .unwrap()
                .split(' ')
                .collect::<Vec<&str>>();

            if auth_header_parts.len() != 2
                || !(auth_header_parts[0].to_lowercase() == "Bearer")
            {
                return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                    .set_message("Missing client bearer token header"));
            }

            let token = auth_header_parts[1];

            let jwk_set = match self.fetch_jwk() {
                Ok(jwk_set) => jwk_set,
                Err(_) => {
                    return Ok(HandlerStatus::new(Code::SERVER_ERROR)
                        .set_message("Unable to fetch JWKs"))
                }
            };

            let header = match decode_header(token) {
                Ok(jwt_header) => jwt_header,
                Err(_) => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("Malformed JWT header"))
                }
            };

            let kid = match header.kid {
                Some(kid) => kid,
                None => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("JWT is missing kid"))
                }
            };

            let matching_jwk = match jwk_set.find(&kid) {
                Some(matching_jwk) => matching_jwk,
                None => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("No matching JWK for kid"))
                }
            };
            let decoding_key = match &matching_jwk.algorithm {
                AlgorithmParameters::RSA(rsa_params) => {
                    match DecodingKey::from_rsa_components(&rsa_params.n, &rsa_params.e) {
                        Ok(decoding_key) => decoding_key,
                        Err(_) => {
                            return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                                .set_message("Malformed RSA key"))
                        }
                    }
                }
                _ => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("Unsupported JWT algorithm"))
                }
            };

            let validation = Validation::new(Algorithm::RS256);
            let token_data = match decode::<Value>(token, &decoding_key, &validation) {
                Ok(token_data) => token_data,
                Err(_) => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR).set_message("Invalid JWT"))
                }
            };

            let claims = token_data.claims;
            let (request_path, method) = match (&request.path, &request.http_method) {
                (None, _) => {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("Missing request path"))
                }
                (Some(path), method) => (path, method),
            };

            if self.config.get().scope_verification {
                if let Err(_) = self.validate_scope(&request_path, &method.to_string(), &claims)
                {
                    return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                        .set_message("Invalid scope for token"));
                }
            }

            if let Err(_) = self.validate_aud(&claims) {
                return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                    .set_message("Invalid audience for token"));
            }

            if let Err(_) = self.validate_iss(&claims) {
                return Ok(HandlerStatus::new(Code::CLIENT_ERROR)
                    .set_message("Invalid issuer for token"));
            }

            if let Err(_) = self.validate_exp(&claims) {
                return Ok(HandlerStatus::new(Code::CLIENT_ERROR).set_message("Expired token"));
            }

            Ok(HandlerStatus::new(Code::OK))
        } else {
            return Ok(HandlerStatus::new(Code::CLIENT_ERROR).set_message("Missing JWT"));
        }
    }
}

#[cfg(test)]
mod test {
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use base64::Engine;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequest;
    use lambda_http::http::HeaderValue;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::RsaPrivateKey;
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::fs::File;
    use idem_handler::exchange::Exchange;
    use idem_handler::handler::Handler;
    use idem_handler::status::Code;
    use idem_handler_config::config::{Config, DefaultConfigProvider};
    use crate::handler::jwt::{JwkProvider, JwtValidationHandler, JwtValidationHandlerConfig};
    use crate::handler::LambdaExchange;

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
        let test_file = File::open("./src/handler/jwt/test/public_private_keypair.json");
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

    #[tokio::test(flavor = "current_thread")]
    async fn test_valid_jwt_validator_handler() {
        // generate a valid token from a test pub private key set
        let token = get_test_key_gen();
        let complete_token_header = format!("{} {}", "Bearer", token);

        // create request containing our valid jwt and execute the handler
        let mut test_request = ApiGatewayProxyRequest::default();
        test_request.headers.insert(
            "Authorization",
            HeaderValue::from_str(&complete_token_header).unwrap(),
        );
        let mut test_exchange: LambdaExchange = Exchange::new();
        test_exchange.save_input(test_request);
        let jwt_validation_handler =
            JwtValidationHandler::init_handler(Config::new(DefaultConfigProvider).unwrap());

        // make sure the result is OK
        let result = jwt_validation_handler
            .exec(&mut test_exchange)
            .await
            .unwrap();
        let result_code = result.code();
        if result_code.any_flags(Code::OK) {
            assert!(
                true,
                "Handler returned an OK status meaning validation passed"
            )
        } else {
            assert!(
                false,
                "Handler returned something other than OK status meaning validation did no pass"
            )
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_invalid_jwt_validator_handler() {
        // An invalid/malformed JWT token
        let invalid_token = "Bearer 389475983475893745invalid_jwt4789234789";

        // Create an exchange containing the header with our invalid token.
        let mut test_request = ApiGatewayProxyRequest::default();
        test_request.headers.insert(
            "Authorization",
            HeaderValue::from_str(&invalid_token).unwrap(),
        );
        let mut test_exchange: LambdaExchange = Exchange::new();
        test_exchange.save_input(test_request);

        // execute the validation and get the result
        let jwt_validation_handler =
            JwtValidationHandler::init_handler(Config::new(DefaultConfigProvider).unwrap());
        let result = jwt_validation_handler
            .exec(&mut test_exchange)
            .await
            .unwrap();

        // make sure we returned the client error code with the Malformed 'JWT header message'
        let result_code = result.code();
        let result_message = result.message();
        if result_code.any_flags(Code::CLIENT_ERROR) && result_message == "Malformed JWT header" {
            assert!(true)
        } else {
            assert!(false)
        }
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
        assert!(jwk_set.keys.iter().any(|jwk| jwk.clone().common.key_id.unwrap() == "DDbt045YVtnjCkzHUv-rFN4wPfGD3Upk9_da_yweZ1c"));
    }
}



