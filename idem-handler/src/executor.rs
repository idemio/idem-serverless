use crate::handler::Handler;

pub struct HandlerExecutor<Input, Output, Metadata> {
    pub handlers: Vec<Box<dyn Handler<Input, Output, Metadata> + Send>>,
}

impl<Input, Output, Metadata> HandlerExecutor<Input, Output, Metadata>
where
    Input: Default + Send,
    Output: Default + Send,
    Metadata: Send,
{
    pub fn new() -> HandlerExecutor<Input, Output, Metadata> {
        HandlerExecutor {
            handlers: Vec::new()
        }
    }

    // TODO - change this so you can 'filter' different handler chains (i.e. filter by path, ip, content, etc.)
    pub fn add_handler(&mut self, handler: Box<dyn Handler<Input, Output, Metadata> + Send>) -> &mut Self {
        self.handlers.push(handler);
        self
    }
}
