mod config;
mod http_tracker;
mod udp_tracker;

use hanekawa_common::Config;
use http_tracker::tracker;

use axum::Router;

async fn start_http(cfg: Config) {
    let tracker = tracker(&cfg).await;
    let app = Router::new().nest("/", tracker);

    axum::Server::bind(&([127, 0, 0, 1], 8001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn start_udp() {
    udp_tracker::start().await;
}

pub async fn start() {
    let _ = dotenvy::dotenv();

    let cfg = crate::config::load_config();

    let hh = tokio::spawn(start_http(cfg.clone()));
    let uh = tokio::spawn(start_udp());

    let _ = tokio::join!(hh, uh);
}
