use super::proto::{
    AnnounceRequest, AnnounceResponse, PeerData, PeerScrapeData, ScrapeRequest, ScrapeResponse,
};

use hanekawa_common::types::{Event, Peer};
use hanekawa_storage::{PeerRepository, UpdatePeerAnnounceCommand};

use std::net::IpAddr;

#[derive(Debug)]
struct InfoHashData {
    peer_id: String,
    complete: u32,
    downloaded: u32,
    incomplete: u32,
}

#[derive(Clone)]
pub struct HttpTrackerService {
    repository: PeerRepository,
}

impl HttpTrackerService {
    pub async fn new(repository: PeerRepository) -> Self {
        Self { repository }
    }

    pub async fn announce(&self, announce: AnnounceRequest) -> AnnounceResponse {
        let cmd = UpdatePeerAnnounceCommand {
            info_hash: announce.info_hash,
            peer_id: announce.peer_id,
            ip: announce.ip,
            port: announce.port,
            uploaded: announce.uploaded,
            downloaded: announce.downloaded,
            left: announce.left,
            event: announce.event,
            last_update_ts: time::OffsetDateTime::now_utc(),
        };

        let peers = self.repository.update_peer_announce(&cmd).await;

        let is_compact = announce.compact.unwrap_or(1) == 1;
        let (peers, peers6) = encode_peers(peers, is_compact);

        AnnounceResponse {
            interval: 60,
            peers,
            peers6,
        }
    }

    pub async fn scrape(&self, _request: ScrapeRequest) -> ScrapeResponse {
        let datas: Vec<InfoHashData> = vec![];

        let files = datas
            .into_iter()
            .map(|data| {
                (
                    data.peer_id,
                    PeerScrapeData {
                        complete: data.complete,
                        downloaded: data.downloaded,
                        incomplete: data.incomplete,
                    },
                )
            })
            .collect();

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
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn ipv4_peer() -> Peer {
        Peer {
            peer_id: "012345678901234567890".to_string(),
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 5005,
        }
    }

    fn ipv6_peer() -> Peer {
        Peer {
            peer_id: "09876543210987654321".to_string(),
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
