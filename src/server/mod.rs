use std::net::Ipv4Addr;

use axum::extract::Query;
use axum::routing::get;
use axum::Router;

use crate::bencode::{encode, Value};
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
    ip: Ipv4Addr,
    port: u16,
}

async fn announce(Query(announce): Query<AnnounceRequest>) -> String {
    let peers: Vec<Peer> = vec![];

    if announce.compact.unwrap_or(1) == 1 {
        // BEP 23 Compact representation
        use bytes::{BufMut, BytesMut};
        use std::collections::BTreeMap;

        let mut peer_string = BytesMut::new();
        for peer in peers.into_iter() {
            let ip_bytes: u32 = peer.ip.into();
            peer_string.put_u32(ip_bytes);
            peer_string.put_u16(peer.port)
        }
        let peers = std::str::from_utf8(&peer_string).unwrap().to_string();

        let mut data = BTreeMap::new();
        data.insert("interval".to_string(), Value::Int(30));
        data.insert("peers".to_string(), Value::String(peers));

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
                data.insert("port".to_string(), Value::Int(p.port as i32));

                Value::Dict(data)
            })
            .collect();

        let mut data = BTreeMap::new();
        data.insert("interval".to_string(), Value::Int(30));
        data.insert("peers".to_string(), Value::List(peer_dicts));

        encode(&Value::Dict(data))
    }
}

pub async fn start() {
    let app = Router::new().route("/announce", get(announce));

    axum::Server::bind(&([127, 0, 0, 1], 8001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
