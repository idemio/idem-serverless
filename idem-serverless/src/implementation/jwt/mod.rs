pub mod config;
pub mod handler;
mod jwk_provider;

pub(crate) const BEARER_PREFIX: &str = "bearer";
pub(crate) const AUTH_HEADER_NAME: &str = "authorization";
