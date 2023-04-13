use axum::extract::Query;
use axum::routing::get;
use axum::Router;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Event {
    Started,
    Completed,
    Stopped,
}

#[derive(Debug, serde::Deserialize)]
struct AnnounceRequest {
    info_hash: String,
    peer_id: String,
    ip: Option<String>,
    port: u16,
    uploaded: usize,
    left: usize,
    event: Option<Event>,
}

async fn announce(Query(announce): Query<AnnounceRequest>) {}

pub async fn start() {
    let app = Router::new().route("/announce", get(announce));

    axum::Server::bind(&([127, 0, 0, 1], 8001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
