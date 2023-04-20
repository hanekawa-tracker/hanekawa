use super::proto::{
    AnnounceRequest, AnnounceResponse, Peer, PeerData, PeerScrapeData, ScrapeRequest,
    ScrapeResponse,
};

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug)]
struct InfoHashData {
    peer_id: String,
    complete: u32,
    downloaded: u32,
    incomplete: u32,
}

pub struct HttpTrackerService();

impl HttpTrackerService {
    pub async fn announce(&self, announce: AnnounceRequest) -> AnnounceResponse {
        let peers: Vec<Peer> = vec![];

        let (peers, peers6) = if announce.compact.unwrap_or(1) == 1 {
            // BEP 23 Compact representation
            use bytes::{BufMut, BytesMut};

            let mut peer_string = BytesMut::new();
            let mut peer6_string = BytesMut::new();
            for peer in peers.into_iter() {
                match peer.ip {
                    IpAddr::V4(ip) => {
                        let ip_bytes: u32 = ip.into();
                        peer_string.put_u32(ip_bytes);
                        peer_string.put_u16(peer.port);
                    }
                    IpAddr::V6(ip) => {
                        let ip_bytes: u128 = ip.into();
                        peer6_string.put_u128(ip_bytes);
                        peer6_string.put_u16(peer.port);
                    }
                }
            }

            (
                PeerData::Compact(peer_string.to_vec()),
                PeerData::Compact(peer6_string.to_vec()),
            )
        } else {
            // BEP 3 representation
            let (peers, peers6) = peers.into_iter().partition::<Vec<_>, _>(|p| p.ip.is_ipv4());

            (PeerData::Long(peers), PeerData::Long(peers6))
        };

        AnnounceResponse {
            interval: 60,
            peers,
            peers6,
        }
    }

    pub async fn scrape(&self, _request: ScrapeRequest) -> ScrapeResponse {
        let datas: Vec<InfoHashData> = vec![];

        let mut files = HashMap::new();

        for data in datas.into_iter() {
            files.insert(
                data.peer_id,
                PeerScrapeData {
                    complete: data.complete,
                    downloaded: data.downloaded,
                    incomplete: data.incomplete,
                },
            );
        }

        ScrapeResponse { files }
    }
}
