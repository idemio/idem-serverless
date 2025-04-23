use std::env;
use std::str::FromStr;
use lambda_http::{lambda_runtime, service_fn, tracing, Error};
mod executor;
mod exchange;
mod handlers;
mod config;
mod status;

use executor::entry;

fn main() -> Result<(), Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing::init_default_subscriber();
            lambda_runtime::run(service_fn(entry)).await
        })
}
