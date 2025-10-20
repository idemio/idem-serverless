use core::result::Result;
use lambda_http::{lambda_runtime, service_fn, tracing, Error};
mod handler;
mod entry;

use entry::entry;
pub const ROOT_CONFIG_PATH: &str = "/opt/config";

fn main() -> Result<(), Error> {

//    if let Err(_) = init_or_replace_config(format!("{}/{}", ROOT_CONFIG_PATH, "handlers.json").as_str()) {
//        panic!("Failed to load config");
//    }


    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing::init_default_subscriber();
            lambda_runtime::run(service_fn(entry)).await
        })
}
