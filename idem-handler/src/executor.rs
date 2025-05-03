use std::sync::Arc;
use idem_config::execution_flow_config::ExecutionFlowConfig;
use crate::handler::Handler;

pub struct HandlerExecutor<Input, Output, Metadata> {
    execution_flow_config: ExecutionFlowConfig,
    pub handlers: Vec<Arc<dyn Handler<Input, Output, Metadata> + Send>>,
}

impl<Input, Output, Metadata> HandlerExecutor<Input, Output, Metadata>
where
    Input: Default + Send,
    Output: Default + Send,
    Metadata: Send,
{
    pub fn new(config: ExecutionFlowConfig) -> HandlerExecutor<Input, Output, Metadata> {
        HandlerExecutor {
            execution_flow_config: config,
            handlers: Vec::new()
        }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn Handler<Input, Output, Metadata> + Send>) -> &mut Self {
        self.handlers.push(handler);
        self
    }



    pub fn handle(&self, path: &str, input: Input, metadata: Metadata) {

    }
}
