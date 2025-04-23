use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub type ConfigResult<T> = Result<T, Box<dyn Error>>;

pub enum LoadMethod {
    Remote,
    Programmatically,
    LocalFile,
    Default,
}

#[derive(Debug)]
pub struct LoadMethodError {
    method: String
}

impl Display for LoadMethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid 'LoadMethod' {} provided.", self.method)
    }
}

impl Error for LoadMethodError {}

impl FromStr for LoadMethod {
    type Err = LoadMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "remote" => Ok(LoadMethod::Remote),
            "programmatically" => Ok(LoadMethod::Programmatically),
            "localfile" => Ok(LoadMethod::LocalFile),
            "default" => Ok(LoadMethod::Default),
            _ => Err(LoadMethodError { method: s.to_owned() }),
        }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    MalformedConfig,
    MissingConfig,
    EmptyConfig,
}

pub trait Config
where
    Self: Sized + Default
{
    fn load(method: LoadMethod) -> ConfigResult<Self>
    {
        match method {
            LoadMethod::Remote => Self::load_remote(),
            LoadMethod::Programmatically => Self::load_programmatically(),
            LoadMethod::LocalFile => Self::load_local_file(),
            LoadMethod::Default => Ok(Self::default()),
        }
    }
    fn load_local_file() -> ConfigResult<Self>;
    fn load_programmatically() -> ConfigResult<Self>;
    fn load_remote() -> ConfigResult<Self>;
}

#[derive(Debug)]
pub(crate) struct LocalFileConfigError {
    path: String,
    filename: String,
    error_type: ErrorType,
}

impl Display for LocalFileConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            ErrorType::MalformedConfig => write!(f, "File {} found under the path {} is malformed.", self.filename, self.path),
            ErrorType::MissingConfig => write!(f, "File {} found under the path {} cannot be found.", self.filename, self.path),
            ErrorType::EmptyConfig => write!(f, "File {} found under the path {} is either empty or not a file.", self.filename, self.path)

        }
    }
}

impl Error for LocalFileConfigError {}

#[derive(Debug)]
pub(crate) struct RemoteConfigError;

impl Display for RemoteConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!("impl display for RemoteConfigError")
    }
}

impl Error for RemoteConfigError {}

#[derive(Debug)]
pub(crate) struct ProgrammaticallyConfigError;

impl Display for ProgrammaticallyConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!("impl display for ProgrammaticallyConfigError")
    }
}

impl Error for ProgrammaticallyConfigError {}
