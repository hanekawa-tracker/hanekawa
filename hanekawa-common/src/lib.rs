pub mod types;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub peer_announce_interval: u32,
    pub peer_activity_timeout: u32,
}
