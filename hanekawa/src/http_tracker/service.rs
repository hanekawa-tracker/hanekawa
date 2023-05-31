use super::proto::{AnnounceRequest, AnnounceResponse, PeerData, ScrapeRequest, ScrapeResponse};

use hanekawa_common::{
    types::{InfoHashStatus, Peer},
    repository::{info_hash::{GetInfoHashSummary, InfoHashRepository}, peer::{PeerRepository, UpdatePeerAnnounce, GetPeers, GetPeerStatistics}},
    Config,
};

use std::net::IpAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct HttpTrackerService {
    config: Config,
    peer_repository: Arc<dyn PeerRepository>,
    info_hash_repository: Arc<dyn InfoHashRepository>,
}

impl HttpTrackerService {
    pub async fn new(
        config: &Config,
        peer_repository: Arc<dyn PeerRepository>,
        info_hash_repository: Arc<dyn InfoHashRepository>,
    ) -> Self {
        Self {
            config: config.clone(),
            peer_repository,
            info_hash_repository,
        }
    }

    pub async fn announce(&self, announce: AnnounceRequest, sender_ip: IpAddr) -> AnnounceResponse {
        let info_hash_summary = self
            .info_hash_repository
            .get_info_hash_summary(GetInfoHashSummary {
                info_hash: &announce.info_hash,
            })
            .await
            .unwrap();

        if info_hash_summary.status == InfoHashStatus::ExplicitDeny
            || (self.config.only_allowed_info_hashes
                && info_hash_summary.status != InfoHashStatus::ExplicitAllow)
        {
            let st = info_hash_summary.info_hash.to_hex();
            dbg!(st);
            unimplemented!("denied info_hash");
        }

        let cmd = UpdatePeerAnnounce {
            info_hash: &announce.info_hash,
            peer_id: &announce.peer_id,
            ip: sender_ip,
            port: announce.port,
            uploaded: announce.uploaded,
            downloaded: announce.downloaded,
            left: announce.left,
            event: announce.event,
            update_timestamp: time::OffsetDateTime::now_utc(),
        };

        self.peer_repository.update_peer_announce(cmd)
            .await
            .unwrap();

        let active_after = time::OffsetDateTime::now_utc() - std::time::Duration::from_secs(self.config.peer_activity_timeout as u64);

        let peers = self.peer_repository.get_peers(GetPeers {
            info_hash: &announce.info_hash,
            active_after: Some(active_after)
        })
            .await
            .unwrap();

        let peers = peers
            .into_iter()
            .filter(|p| p.ip != sender_ip)
            .collect();

        let is_compact = announce.compact.unwrap_or(1) == 1;
        let (peers, peers6) = encode_peers(peers, is_compact);

        AnnounceResponse {
            interval: self.config.peer_announce_interval,
            peers,
            peers6,
        }
    }

    pub async fn scrape(&self, request: ScrapeRequest) -> ScrapeResponse {
        let cmd = GetPeerStatistics {
            info_hashes: &request.info_hash,
        };

        let files = self.peer_repository.get_peer_statistics(cmd)
            .await
            .unwrap();

        ScrapeResponse { files }
    }
}

fn encode_peers(peers: Vec<Peer>, is_compact: bool) -> (PeerData, PeerData) {
    if is_compact {
        use bytes::{BufMut, BytesMut};

        let mut peers_bytes = BytesMut::new();
        let mut peers6_bytes = BytesMut::new();

        for peer in peers {
            match peer.ip {
                IpAddr::V4(ip) => {
                    let ip_bytes: u32 = ip.into();
                    peers_bytes.put_u32(ip_bytes);
                    peers_bytes.put_u16(peer.port);
                }
                IpAddr::V6(ip) => {
                    let ip_bytes: u128 = ip.into();
                    peers6_bytes.put_u128(ip_bytes);
                    peers6_bytes.put_u16(peer.port);
                }
            }
        }

        (
            PeerData::Compact(peers_bytes.to_vec()),
            PeerData::Compact(peers6_bytes.to_vec()),
        )
    } else {
        let (peers, peers6) = peers.into_iter().partition::<Vec<_>, _>(|p| p.ip.is_ipv4());

        (PeerData::Long(peers), PeerData::Long(peers6))
    }
}

#[cfg(test)]
mod test {
    use hanekawa_common::types::PeerId;

    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn ipv4_peer() -> Peer {
        Peer {
            peer_id: PeerId("012345678901234567890".as_bytes().to_vec()),
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 5005,
        }
    }

    fn ipv6_peer() -> Peer {
        Peer {
            peer_id: PeerId("09876543210987654321".as_bytes().to_vec()),
            ip: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            port: 5005,
        }
    }

    #[test]
    fn encodes_compact_peers_if_compact() {
        let peers = vec![ipv4_peer(), ipv6_peer()];

        let result = encode_peers(peers, true);

        let bs4 = vec![127, 0, 0, 1, 19, 141];
        let bs6 = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 19, 141];
        assert_eq!((PeerData::Compact(bs4), PeerData::Compact(bs6)), result);
    }

    #[test]
    fn encodes_noncompact_peers_if_noncompact() {
        let peers = vec![ipv4_peer(), ipv6_peer()];

        let result = encode_peers(peers, false);

        assert_eq!(
            (
                PeerData::Long(vec![ipv4_peer()]),
                PeerData::Long(vec![ipv6_peer()])
            ),
            result
        );
    }
}
