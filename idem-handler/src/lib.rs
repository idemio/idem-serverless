use std::pin::Pin;
use crate::status::{HandlerExecutionError, HandlerStatus};

pub mod handler;
pub mod status;
pub mod exchange;
pub mod executor;
pub mod factory;

pub type HandlerOutput<'a> = Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'a>>;
