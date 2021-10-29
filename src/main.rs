#![forbid(unsafe_code)]
extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

use std::sync::Arc;

use reqwest::Client;
use rocket::tokio::sync::RwLock;
use rocket::{tokio, Config, Error};

use crate::configuration::TidvattenConfig;
use crate::remote::{refresh_keepers, Keepers};

mod api;
mod configuration;
mod remote;

#[rocket::main]
async fn main() -> Result<(), Error> {
    let api_base = "/api/v1/";

    let keepers_registry = Arc::new(RwLock::new(Keepers::new()));
    let cloned_keepers_registry = keepers_registry.clone();

    let app_config = Arc::new(
        Config::figment()
            .extract::<TidvattenConfig>()
            .expect("There is no configuration for app!"),
    );
    let app_config_cloned = app_config.clone();

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let http_client = Client::builder()
        .gzip(true)
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to create HTTP client!");

    tokio::spawn(async move {
        info!(
            "Spawned future to maintain keepers registry every {:?}",
            app_config_cloned.tasks.keepers
        );
        let mut interval = tokio::time::interval(app_config_cloned.tasks.keepers);

        loop {
            refresh_keepers(&http_client, &cloned_keepers_registry, &app_config_cloned).await;
            interval.tick().await;
        }
    });

    rocket::build()
        .manage(app_config)
        .mount(api_base, api::routes())
        .register(api_base, api::catchers())
        .launch()
        .await
}
