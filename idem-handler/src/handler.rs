use std::pin::Pin;
use crate::exchange::Exchange;
use crate::status::{HandlerExecutionError, HandlerStatus};

pub trait Handler<Input, Output, Metadata>: Send
where
    Input: Default + Send,
    Output: Default + Send,
    Metadata: Send,
{
    fn process<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut Exchange<Input, Output, Metadata>,
    ) -> Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'result>>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result;

}