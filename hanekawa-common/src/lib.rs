pub mod repository;
pub mod types;

use std::net::Ipv4Addr;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_ip: Ipv4Addr,
    pub http_bind_port: u16,
    pub udp_bind_port: u16,
    pub peer_announce_interval: u32,
    pub peer_activity_timeout: u32,
    pub only_allowed_info_hashes: bool,
}
