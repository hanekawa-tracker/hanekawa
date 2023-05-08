use hanekawa_common::types::{Event, Peer};

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, serde::Deserialize)]
pub struct AnnounceRequest {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: IpAddr,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: Option<Event>,
    pub compact: Option<u8>,
}

#[derive(serde::Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PeerData {
    Compact(#[serde(with = "serde_bytes")] Vec<u8>),
    Long(Vec<Peer>),
}

#[derive(serde::Serialize)]
pub struct AnnounceResponse {
    pub interval: u32,
    pub peers: PeerData,
    pub peers6: PeerData,
}

#[derive(Debug, serde::Deserialize)]
pub struct ScrapeRequest {
    pub info_hash: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PeerScrapeData {
    pub complete: u32,
    pub downloaded: u32,
    pub incomplete: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct ScrapeResponse {
    pub files: HashMap<String, PeerScrapeData>,
}
