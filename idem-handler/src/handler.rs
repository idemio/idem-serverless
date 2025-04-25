use std::pin::Pin;
use crate::exchange::Exchange;
use crate::status::{HandlerExecutionError, HandlerStatus};

pub trait Handler<I, O, M>: Send
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<I, O, M>,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}

pub trait HandlerLoader<I, O, M>: Send
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
    Self: Sized,
{
    type Err;
    fn async_from_str<'i1, 'o>(
        s: &'i1 str,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Handler<I, O, M> + Send>, Self::Err>> + Send + 'o>>
    where
        'i1: 'o,
        Self: 'o;
}