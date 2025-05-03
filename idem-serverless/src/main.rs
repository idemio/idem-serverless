use lambda_http::{lambda_runtime, service_fn, tracing, Error};
mod implementation;
mod entry;

use entry::entry;
use idem_config::config_cache::{init_or_replace_config};
pub const ROOT_CONFIG_PATH: &str = "/opt/config";

fn main() -> Result<(), Error> {

    if let Err(_) = init_or_replace_config(format!("{}/{}", ROOT_CONFIG_PATH, "handlers.json").as_str()) {
        panic!("Failed to load config");
    }


    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing::init_default_subscriber();
            lambda_runtime::run(service_fn(entry)).await
        })
}
