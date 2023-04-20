use std::collections::HashMap;
use std::net::IpAddr;

use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query as MultiQuery;

use crate::bencode;
use crate::types::Event;
use bytes::Bytes;

#[derive(Debug, serde::Deserialize)]
struct AnnounceRequest {
    info_hash: String,
    peer_id: String,
    ip: Option<String>,
    port: u16,
    uploaded: usize,
    left: usize,
    event: Option<Event>,
    compact: Option<u8>,
}

#[derive(serde::Serialize)]
struct Peer {
    #[serde(rename = "peer id")]
    peer_id: String,
    ip: IpAddr,
    port: u16,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum PeerData {
    Compact(#[serde(with = "serde_bytes")] Vec<u8>),
    Long(Vec<Peer>),
}

#[derive(serde::Serialize)]
struct AnnounceResponse {
    interval: u32,
    peers: PeerData,
    peers6: PeerData,
}

async fn announce(Query(announce): Query<AnnounceRequest>) -> Bytes {
    let peers: Vec<Peer> = vec![Peer {
        peer_id: "123".to_string(),
        ip: IpAddr::V4(std::net::Ipv4Addr::new(132, 216, 25, 20)),
        port: 8001,
    }];

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

    bencode::to_bytes(&AnnounceResponse {
        interval: 60,
        peers,
        peers6,
    })
    .unwrap()
}

#[derive(Debug, serde::Deserialize)]
struct ScrapeRequest {
    info_hash: Vec<String>,
}

#[derive(Debug)]
struct InfoHashData {
    peer_id: String,
    complete: u32,
    downloaded: u32,
    incomplete: u32,
}

#[derive(Debug, serde::Serialize)]
struct PeerScrapeData {
    complete: u32,
    downloaded: u32,
    incomplete: u32,
}

#[derive(Debug, serde::Serialize)]
struct ScrapeResponse {
    files: HashMap<String, PeerScrapeData>,
}

// BEP 48: Tracker Protocol Extension: Scrape
async fn scrape(MultiQuery(_scrape): MultiQuery<ScrapeRequest>) -> Bytes {
    let datas: Vec<InfoHashData> = vec![InfoHashData {
        peer_id: "testerllalal".to_string(),
        complete: 32,
        downloaded: 42,
        incomplete: 17,
    }];

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

    bencode::to_bytes(&ScrapeResponse { files }).unwrap()
}

pub async fn start() {
    let app = Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape));

    axum::Server::bind(&([127, 0, 0, 1], 8001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
