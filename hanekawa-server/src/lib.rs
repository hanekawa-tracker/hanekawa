mod admin;
mod config;
mod http;
mod http_tracker;
mod udp_tracker;

use hanekawa_common::Config;
use http_tracker::tracker;

use axum::Router;
use tokio_util::sync::CancellationToken;

async fn start_http(cfg: Config, kt: CancellationToken) {
    let tracker = tracker(&cfg).await;
    let admin = admin::admin(&cfg).await;

    let app = Router::new().nest("/", tracker).nest("/admin", admin);

    axum::Server::bind(&(cfg.bind_ip, cfg.http_bind_port).into())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(async { kt.cancelled().await })
        .await
        .unwrap();
}

async fn start_udp(cfg: Config, kt: CancellationToken) {
    udp_tracker::start(&cfg, kt).await;
}

pub async fn start() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let cfg = crate::config::load_config();

    tracing::event!(tracing::Level::DEBUG, "yay");

    let kt = tokio_util::sync::CancellationToken::new();

    let hh = tokio::spawn(start_http(cfg.clone(), kt.child_token()));
    let uh = tokio::spawn(start_udp(cfg.clone(), kt.child_token()));

    let cancel = tokio::spawn(async move {
        use tokio::signal::{
            ctrl_c,
            unix::{signal, SignalKind},
        };

        let mut int = signal(SignalKind::interrupt()).unwrap();
        let mut term = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = int.recv() => {},
            _ = term.recv() => {},
            _ = ctrl_c() => {}
        }

        tracing::info!("Shutting down...");
        kt.cancel();
    });

    let _ = tokio::join!(cancel, hh, uh);
}
