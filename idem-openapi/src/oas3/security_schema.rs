use crate::oas3::spec_extensions;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        name: String,
        #[serde(rename = "in")]
        location: String,
    },
    #[serde(rename = "http")]
    Http {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        scheme: String,
        #[serde(rename = "bearerFormat")]
        bearer_format: Option<String>,
    },

    /// OAuth2 authentication.
    #[serde(rename = "oauth2")]
    OAuth2 {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        flows: Flows,
    },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(rename = "openIdConnectUrl")]
        open_id_connect_url: String,
    },
    #[serde(rename = "mutualTLS")]
    MutualTls {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Flows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<PasswordFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<ClientCredentialsFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeFlow>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImplicitFlow {
    pub authorization_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,
    #[serde(default)]
    pub scopes: HashMap<String, String>,
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PasswordFlow {
    pub token_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,
    #[serde(default)]
    pub scopes: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCredentialsFlow {
    pub token_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,
    #[serde(default)]
    pub scopes: HashMap<String, String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCodeFlow {
    pub authorization_url: Url,
    pub token_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,
    #[serde(default)]
    pub scopes: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Callback(
    /// A Path Item Object used to define a callback request and expected responses.
    serde_json::Value, // TODO: Add "Specification Extensions" https://spec.openapis.org/oas/v3.1.1#specificationExtensions}
);

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_http_basic_deser() {
        const HTTP_BASIC_SAMPLE: &str = r#"{"type": "http", "scheme": "basic"}"#;
        let obj: SecurityScheme = serde_json::from_str(HTTP_BASIC_SAMPLE).unwrap();

        assert!(matches!(
            obj,
            SecurityScheme::Http {
                description: None,
                scheme,
                bearer_format: None,
            } if scheme == "basic"
        ));
    }

    #[test]
    fn test_security_scheme_oauth_deser() {
        const IMPLICIT_OAUTH2_SAMPLE: &str = r#"{
          "type": "oauth2",
          "flows": {
            "implicit": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            },
            "authorizationCode": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "tokenUrl": "https://example.com/api/oauth/token",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            }
          }
        }"#;

        let obj: SecurityScheme = serde_json::from_str(IMPLICIT_OAUTH2_SAMPLE).unwrap();
        match obj {
            SecurityScheme::OAuth2 {
                description: _,
                flows,
            } => {
                assert!(flows.implicit.is_some());
                let implicit = flows.implicit.unwrap();
                assert_eq!(
                    implicit.authorization_url,
                    Url::parse("https://example.com/api/oauth/dialog").unwrap()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));

                assert!(flows.authorization_code.is_some());
                let auth_code = flows.authorization_code.unwrap();
                assert_eq!(
                    auth_code.authorization_url,
                    Url::parse("https://example.com/api/oauth/dialog").unwrap()
                );
                assert_eq!(
                    auth_code.token_url,
                    Url::parse("https://example.com/api/oauth/token").unwrap()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));
            }
            _ => panic!("wrong security scheme type"),
        }
    }
}