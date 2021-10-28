use std::time::Duration;

use rocket::serde::{Deserialize, Deserializer};

fn deserialize_tokio_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct TasksConfig {
    #[serde(deserialize_with = "deserialize_tokio_interval")]
    pub keepers: Duration,
}

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct TidvattenConfig {
    pub remote_api_base: String,
    pub tasks: TasksConfig,
}
