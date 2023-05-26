mod encode;

use encode::Bencode;

use hanekawa::http_tracker::proto::{
    AnnounceRequest, AnnounceResponse, ScrapeRequest, ScrapeResponse,
};
use hanekawa::http_tracker::HttpTrackerService;

use axum::extract::{ConnectInfo, Query, State};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query as MultiQuery;
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
    MultiQuery(scrape): MultiQuery<ScrapeRequest>,
    State(tracker): State<HttpTrackerService>,
) -> Bencode<ScrapeResponse> {
    let response = tracker.scrape(scrape).await;
    Bencode(response)
}

pub async fn tracker<S>(cfg: &Config) -> Router<S> {
    let storage = hanekawa_storage::Services::start(cfg).await;

    let tracker = HttpTrackerService::new(cfg, storage.peer, storage.info_hash).await;

    Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape))
        .with_state(tracker)
}
