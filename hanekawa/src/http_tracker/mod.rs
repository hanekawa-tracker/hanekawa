mod encode;
mod proto;
mod service;

use encode::Bencode;
use proto::{AnnounceRequest, AnnounceResponse, ScrapeRequest, ScrapeResponse};

use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query as MultiQuery;

use self::service::HttpTrackerService;

async fn announce(
    Query(announce): Query<AnnounceRequest>,
    State(tracker): State<HttpTrackerService>,
) -> Bencode<AnnounceResponse> {
    let response = tracker.announce(announce).await;
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

pub fn tracker<S>() -> Router<S> {
    let tracker = HttpTrackerService {};

    Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape))
        .with_state(tracker)
}
