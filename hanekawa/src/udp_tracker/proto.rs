pub use super::extensions::Extension;
use crate::types::Event;

#[derive(Debug, Eq, PartialEq)]
pub struct ConnectRequest {
    pub transaction_id: i32,
}

pub struct ConnectResponse {
    pub transaction_id: i32,
    pub connection_id: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AnnounceRequest {
    pub connection_id: i64,
    pub transaction_id: i32,
    pub info_hash: String,
    pub peer_id: String,
    pub downloaded: i64,
    pub left: i64,
    pub uploaded: i64,
    pub event: Option<Event>,
    pub ip_address: Option<i32>,
    pub key: i32,
    pub num_want: Option<i32>,
    pub port: i16,
    pub extensions: Vec<Extension>,
}

pub struct AnnounceResponse {
    pub transaction_id: i32,
    pub interval: i32,
    pub leechers: i32,
    pub seeders: i32,
    pub peers: Vec<(i32, i16)>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScrapeRequest {
    pub connection_id: i64,
    pub transaction_id: i32,
    pub info_hashes: Vec<String>,
}

pub struct InfoHashScrapeData {
    pub seeders: i32,
    pub completed: i32,
    pub leechers: i32,
}

pub struct ScrapeResponse {
    pub transaction_id: i32,
    pub data: Vec<InfoHashScrapeData>,
}

#[derive(Debug)]
pub enum Request {
    Connect(ConnectRequest),
    Announce(AnnounceRequest),
    Scrape(ScrapeRequest),
}

#[derive(Debug)]
pub enum Error {
    Other(()),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Other error")
    }
}

impl std::error::Error for Error {}

pub struct ErrorResponse {
    pub transaction_id: i32,
    pub message: String,
}

pub enum Response {
    Connect(ConnectResponse),
    Announce(AnnounceResponse),
    Scrape(ScrapeResponse),
    Error(ErrorResponse),
}
