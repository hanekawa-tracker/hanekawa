use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Started,
    Completed,
    Stopped,
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct Peer {
    #[serde(rename = "peer id")]
    pub peer_id: String,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct PeerScrapeData {
    pub complete: u32,
    pub downloaded: u32,
    pub incomplete: u32,
}
