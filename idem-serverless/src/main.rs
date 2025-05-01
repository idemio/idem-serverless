use lambda_http::{lambda_runtime, service_fn, tracing, Error};
mod implementation;
mod entry;

use entry::entry;
use idem_config::config_cache::{init_or_replace_config};

fn main() -> Result<(), Error> {
    init_or_replace_config("/opt/config/handlers.json").unwrap();


    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing::init_default_subscriber();
            lambda_runtime::run(service_fn(entry)).await
        })
}
