use derive_more::derive::{Display, Error, From};
use semver::{Error as SemverError, Version};
use crate::oas3::object_ref::RefError;

#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display("Reference error")]
    Ref(RefError),
//    #[display("Schema error")]
//    Schema(SchemaError),
    #[display("Semver error")]
    Semver(SemverError),
    #[display("Unsupported spec file version ({})", _0)]
    UnsupportedSpecFileVersion(#[error(not(source))] Version),
}