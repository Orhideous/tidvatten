use std::collections::HashMap;
use std::sync::Arc;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rocket::serde::Deserialize;
use rocket::tokio::sync::RwLock;

use crate::configuration::TidvattenConfig;

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Keeper {
    pub username: String,
}

pub type Keepers = HashMap<u32, Keeper>;

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct KeeperResponse {
    #[serde(with = "ts_seconds")]
    update_time: DateTime<Utc>,
    result: Keepers,
}

pub type SharedKeepersRegistry = Arc<RwLock<HashMap<u32, Keeper>>>;

async fn fetch_keepers(client: &Client, api_base: &str) -> Result<Keepers, reqwest::Error> {
    let url = format!("{}/static/keepers_user_data", api_base);
    let response = client.get(&url).send().await;
    match response {
        Ok(res) => match res.json::<KeeperResponse>().await {
            Ok(KeeperResponse {
                update_time,
                result,
            }) => {
                debug!("Got new keepers: {:?}", result);
                info!("Received {} keepers from {}", result.len(), update_time);
                Ok(result)
            }
            Err(err) => {
                error!("Failed to deserialize response due to {:?}", err);
                Err(err)
            }
        },
        Err(err) => {
            error!("Failed to fetch keepers from {} due to {:?}", url, err);
            Err(err)
        }
    }
}

pub async fn refresh_keepers(
    http_client: &Client,
    registry: &SharedKeepersRegistry,
    config: &TidvattenConfig,
) {
    let loaded_keepers = fetch_keepers(http_client, &config.remote_api_base).await;
    if let Ok(keepers) = loaded_keepers {
        let mut reg = registry.write().await;
        *reg = keepers;
    }
}
