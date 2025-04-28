use std::fs::File;
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;

pub trait JwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()>;
}

#[derive(Deserialize, Default)]
pub struct LocalJwkProvider {
    file_name: String,
    file_path: String
}

impl JwkProvider for LocalJwkProvider {
    fn jwk(&self) -> Result<JwkSet, ()> {

        let file = match File::open(format!("{}/{}", self.file_path, self.file_name)) {
            Ok(f) => f,
            Err(e) => {
                println!("JWKs file does not exist: {}", e);
                return Err(())
            }
        };
        serde_json::from_reader(file).or(Err(()))
    }
}

#[derive(Deserialize, Default)]
pub struct RemoteJwkProvider {
    jwk_server_url: String,
    jwk_server_path: String
}


#[derive(Deserialize)]
pub enum JwkProviders {

    // TODO - implement remote and other types
    LocalJwkProvider(LocalJwkProvider)
}

impl Default for JwkProviders {
    fn default() -> Self {
        Self::LocalJwkProvider(LocalJwkProvider{
            file_name: String::from("jwks.json"),
            file_path: String::from("./config")
        })
    }
}

impl JwkProvider for JwkProviders {
    fn jwk(&self) -> Result<JwkSet, ()> {
        match self {
            JwkProviders::LocalJwkProvider(local) => local.jwk(),
        }
    }
}