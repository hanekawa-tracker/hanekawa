mod response;

use self::response::OrFailure;

use super::http::encode::Bencode;
use super::http::extractor::Query;

use response::Failure;

use hanekawa::http_tracker::proto::{
    AnnounceRequest, AnnounceResponse, ScrapeRequest, ScrapeResponse,
};
use hanekawa::http_tracker::HttpTrackerService;

use axum::extract::{ConnectInfo, State};
use axum::routing::get;
use axum::Router;
use hanekawa_common::Config;

async fn announce(
    OrFailure(Query(announce)): OrFailure<Query<AnnounceRequest>>,
    State(tracker): State<HttpTrackerService>,
    ConnectInfo(info): ConnectInfo<std::net::SocketAddr>,
) -> Result<Bencode<AnnounceResponse>, Failure> {
    // TODO: extract true source IP from potential proxies.
    let response = tracker.announce(announce, info.ip()).await?;

    Ok(Bencode(response))
}

// BEP 48: Tracker Protocol Extension: Scrape
async fn scrape(
    OrFailure(Query(scrape)): OrFailure<Query<ScrapeRequest>>,
    State(tracker): State<HttpTrackerService>,
) -> Result<Bencode<ScrapeResponse>, Failure> {
    let response = tracker.scrape(scrape).await?;
    Ok(Bencode(response))
}

pub async fn tracker<S>(cfg: &Config) -> Router<S> {
    let storage = hanekawa_storage::Services::start(cfg).await;

    let pr = std::sync::Arc::new(storage.peer);
    let ir = std::sync::Arc::new(storage.info_hash);
    let tracker = HttpTrackerService::new(cfg, pr, ir);

    Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape))
        .with_state(tracker)
}
