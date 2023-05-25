mod config;
mod http_tracker;
mod udp_tracker;

use hanekawa_common::Config;
use http_tracker::tracker;

use axum::Router;

async fn start_http(cfg: Config) {
    let tracker = tracker(&cfg).await;
    let app = Router::new().nest("/", tracker);

    axum::Server::bind(&(cfg.bind_ip, cfg.http_bind_port).into())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .unwrap();
}

async fn start_udp(cfg: Config) {
    udp_tracker::start(&cfg).await;
}

pub async fn start() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let cfg = crate::config::load_config();

    let hh = tokio::spawn(start_http(cfg.clone()));
    let uh = tokio::spawn(start_udp(cfg.clone()));

    let _ = tokio::join!(hh, uh);
}
