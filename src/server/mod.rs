use std::collections::HashMap;
use std::net::IpAddr;

use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query as MultiQuery;

use crate::bencode::{self, encode, Value};
use crate::types::Event;

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

struct Peer {
    peer_id: String,
    ip: IpAddr,
    port: u16,
}

async fn announce(Query(announce): Query<AnnounceRequest>) -> String {
    let peers: Vec<Peer> = vec![];

    if announce.compact.unwrap_or(1) == 1 {
        // BEP 23 Compact representation
        use bytes::{BufMut, BytesMut};
        use std::collections::BTreeMap;

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
        let peers = std::str::from_utf8(&peer_string).unwrap().to_string();
        let peers6 = std::str::from_utf8(&peer6_string).unwrap().to_string();

        let mut data = BTreeMap::new();
        data.insert("interval".to_string(), Value::Int(30));
        data.insert("peers".to_string(), Value::String(peers));
        data.insert("peers6".to_string(), Value::String(peers6));

        encode(&Value::Dict(data))
    } else {
        // BEP 3 representation
        use std::collections::BTreeMap;

        let peer_dicts = peers
            .into_iter()
            .map(|p| {
                let mut data = BTreeMap::new();
                data.insert("peer id".to_string(), Value::String(p.peer_id.clone()));
                data.insert("ip".to_string(), Value::String(p.ip.to_string()));
                data.insert("port".to_string(), Value::Int(p.port as i64));

                Value::Dict(data)
            })
            .collect();

        let mut data = BTreeMap::new();
        data.insert("interval".to_string(), Value::Int(30));
        data.insert("peers".to_string(), Value::List(peer_dicts));

        encode(&Value::Dict(data))
    }
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
    files: HashMap<String, PeerScrapeData>
}

// BEP 48: Tracker Protocol Extension: Scrape
async fn scrape(MultiQuery(_scrape): MultiQuery<ScrapeRequest>) -> String {
    let datas: Vec<InfoHashData> = vec![InfoHashData {
        peer_id: "testerllalal".to_string(),
        complete: 32,
        downloaded: 42,
        incomplete: 17
    }];

    let mut files = HashMap::new();

    for data in datas.into_iter() {
        files.insert(data.peer_id, PeerScrapeData {
            complete: data.complete,
            downloaded: data.downloaded,
            incomplete: data.incomplete
        });
    }

    bencode::to_string(&ScrapeResponse {
        files
    }).unwrap()
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
