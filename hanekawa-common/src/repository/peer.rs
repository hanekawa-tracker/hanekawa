use crate::types::{Event, InfoHash, Peer, PeerId, PeerStatistics};
use std::collections::HashMap;
use std::net::IpAddr;

use time::OffsetDateTime;

use super::Error;

#[derive(Debug, Clone)]
pub struct UpdatePeerAnnounce<'a> {
    pub info_hash: &'a InfoHash,
    pub peer_id: &'a PeerId,
    pub ip: IpAddr,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: Event,
    pub update_timestamp: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct GetPeers<'a> {
    pub info_hash: &'a InfoHash,
    pub active_after: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct GetPeerStatistics<'a> {
    pub info_hashes: &'a [InfoHash],
}

#[async_trait::async_trait]
pub trait PeerRepository: Send + Sync {
    async fn update_peer_announce(&self, cmd: UpdatePeerAnnounce<'_>) -> Result<(), Error>;
    async fn get_peers(&self, cmd: GetPeers<'_>) -> Result<Vec<Peer>, Error>;
    async fn get_peer_statistics(
        &self,
        cmd: GetPeerStatistics<'_>,
    ) -> Result<HashMap<InfoHash, PeerStatistics>, Error>;
}
