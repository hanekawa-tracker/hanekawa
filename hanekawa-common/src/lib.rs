pub mod repository;
pub mod task;
pub mod types;

use std::{net::Ipv4Addr, sync::Arc};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub message_queue_url: String,
    pub bind_ip: Ipv4Addr,
    pub http_bind_port: u16,
    pub udp_bind_port: u16,
    pub peer_announce_interval: u32,
    pub peer_activity_timeout: u32,
    pub only_allowed_info_hashes: bool,
    pub enable_admin_api: bool,
}

impl Config {
    pub fn default_config() -> impl serde::Serialize {
        #[derive(serde::Serialize)]
        struct DefaultConfig {
            pub bind_ip: Ipv4Addr,
            pub http_bind_port: u16,
            pub udp_bind_port: u16,
            pub peer_announce_interval: u32,
            pub peer_activity_timeout: u32,
            pub only_allowed_info_hashes: bool,
            pub enable_admin_api: bool,
        }

        let defaults = DefaultConfig {
            bind_ip: "0.0.0.0".parse().unwrap(),
            http_bind_port: 8001,
            udp_bind_port: 8002,
            peer_announce_interval: 60,
            peer_activity_timeout: 120,
            only_allowed_info_hashes: false,
            enable_admin_api: false,
        };

        defaults
    }
}

#[derive(Clone)]
pub struct Services {
    pub peer_repository: Arc<dyn crate::repository::peer::PeerRepository>,
    pub info_hash_repository: Arc<dyn crate::repository::info_hash::InfoHashRepository>,
    pub task_queue: Arc<dyn crate::task::TaskQueue>,
}
