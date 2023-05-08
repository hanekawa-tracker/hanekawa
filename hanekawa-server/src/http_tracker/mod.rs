mod encode;

use encode::Bencode;

use hanekawa::http_tracker::proto::{
    AnnounceRequest, AnnounceResponse, ScrapeRequest, ScrapeResponse,
};
use hanekawa::http_tracker::HttpTrackerService;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query as MultiQuery;

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

pub async fn tracker<S>() -> Router<S> {
    let repo = hanekawa_storage::PeerRepository::new(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let tracker = HttpTrackerService::new(repo).await;

    Router::new()
        .route("/announce", get(announce))
        .route("/scrape", get(scrape))
        .with_state(tracker)
}
