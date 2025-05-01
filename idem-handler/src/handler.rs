use crate::exchange::Exchange;
use crate::HandlerOutput;

pub trait Handler<Input, Output, Metadata>: Send
where
    Input: Default + Send,
    Output: Default + Send,
    Metadata: Send,
{
    fn exec<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut Exchange<Input, Output, Metadata>,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result;
}