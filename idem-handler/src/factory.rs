use crate::handler::Handler;
use idem_config::config::ProviderType;

pub trait HandlerFactory<Input, Output, Metadata>
where
    Input: Default + Send,
    Output: Default + Send,
    Metadata: Send
{

    type Err;
    type CreatedHandler: Handler<Input, Output, Metadata>;

    fn create_handler(
        name: &str,
        provider_type: ProviderType,
    ) -> Result<Self::CreatedHandler, Self::Err>;
}
