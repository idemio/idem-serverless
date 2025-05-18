use idem_handler_config::config_cache::get_file;
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;
//use idem_config::config_cache::get_file;

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
