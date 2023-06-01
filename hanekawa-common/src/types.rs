use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PeerId(#[serde(with = "serde_bytes")] pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct InfoHash(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl InfoHash {
    pub fn from_hex(s: impl AsRef<str>) -> Self {
        let bytes = hex::decode(s.as_ref()).unwrap();
        Self(bytes)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Started,
    Completed,
    Stopped,
    Interval,
}

impl Default for Event {
    fn default() -> Self {
        Self::Interval
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        match self {
            Self::Started => "started".to_string(),
            Self::Completed => "completed".to_string(),
            Self::Stopped => "stopped".to_string(),
            Self::Interval => "interval".to_string(),
        }
    }
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct Peer {
    #[serde(rename = "peer id")]
    pub peer_id: PeerId,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct PeerStatistics {
    pub complete: u32,
    pub downloaded: u32,
    pub incomplete: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoHashStatus {
    Unknown,
    ExplicitAllow,
    ExplicitDeny,
}

#[derive(Debug)]
pub struct InfoHashSummary {
    pub info_hash: InfoHash,
    pub status: InfoHashStatus,
}
