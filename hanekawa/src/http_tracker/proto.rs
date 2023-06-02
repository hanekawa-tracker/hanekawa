use hanekawa_common::types::{Event, InfoHash, Peer, PeerId, PeerStatistics};

use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub enum Error {
    ServerError(String),
    InfoHashNotAllowed(String),
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerError(s) => f.write_fmt(format_args!("server error: {s}")),
            Self::InfoHashNotAllowed(s) => f.write_fmt(format_args!("info hash not allowed: {s}")),
            Self::Other(s) => f.write_fmt(format_args!("error: {s}")),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AnnounceRequest {
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    #[serde(default)]
    pub event: Event,
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
    pub info_hash: Vec<InfoHash>,
}

#[derive(Debug, serde::Serialize)]
pub struct ScrapeResponse {
    pub files: HashMap<InfoHash, PeerStatistics>,
}
