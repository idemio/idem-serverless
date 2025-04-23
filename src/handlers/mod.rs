use std::future::Future;
use std::pin::Pin;
use crate::exchange::Exchange;

pub(crate) mod echo_test_middleware;
pub(crate) mod invoke_lambda_handler;
mod health_check_handler;
mod traceability_handler;
mod cors_handler;
mod header_handler;

pub trait Handler<I, O, M>: Send
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<I, O, M>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}