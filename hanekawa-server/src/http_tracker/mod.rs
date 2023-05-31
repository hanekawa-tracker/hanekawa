mod encode;
mod extractor;

use encode::Bencode;
use extractor::Query;

use hanekawa::http_tracker::proto::{
    AnnounceRequest, AnnounceResponse, ScrapeRequest, ScrapeResponse,
};
use hanekawa::http_tracker::HttpTrackerService;

use axum::extract::{ConnectInfo, State};
use axum::routing::get;
use axum::Router;
use hanekawa_common::Config;

async fn announce(
    Query(announce): Query<AnnounceRequest>,
    State(tracker): State<HttpTrackerService>,
    ConnectInfo(info): ConnectInfo<std::net::SocketAddr>,
) -> Bencode<AnnounceResponse> {
    // TODO: extract true source IP from potential proxies.
    let response = tracker.announce(announce, info.ip()).await;
    Bencode(response)
}

// BEP 48: Tracker Protocol Extension: Scrape
async fn scrape(
    Query(scrape): Query<ScrapeRequest>,
    State(tracker): State<HttpTrackerService>,
) -> Bencode<ScrapeResponse> {
    let response = tracker.scrape(scrape).await;
    Bencode(response)
}

pub async fn tracker<S>(cfg: &Config) -> Router<S> {
    let storage = hanekawa_storage::Services::start(cfg).await;

    let pr = std::sync::Arc::new(storage.peer);
    let ir = std::sync::Arc::new(storage.info_hash);
    let tracker = HttpTrackerService::new(cfg, pr, ir).await;

    Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape))
        .with_state(tracker)
}
