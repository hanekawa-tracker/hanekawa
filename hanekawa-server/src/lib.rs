mod admin;
mod config;
mod http;
mod http_tracker;
mod task_queue;
mod udp_tracker;

use std::sync::Arc;

use hanekawa_common::{Config, Services};
use http_tracker::tracker;

use axum::Router;
use tokio_util::sync::CancellationToken;

async fn start_http(cfg: Config, services: Services, kt: CancellationToken) {
    let tracker = tracker(&cfg, services).await;
    let admin = admin::admin(&cfg).await;

    let app = Router::new().nest("/", tracker).nest("/admin", admin);

    axum::Server::bind(&(cfg.bind_ip, cfg.http_bind_port).into())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(async { kt.cancelled().await })
        .await
        .unwrap();
}

async fn start_udp(cfg: Config, _services: Services, kt: CancellationToken) {
    udp_tracker::start(&cfg, kt).await;
}

pub async fn start() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let cfg = crate::config::load_config();

    let kt = tokio_util::sync::CancellationToken::new();

    let storage = hanekawa_storage::Services::start(&cfg).await;
    let queue_conn = hanekawa_queue::QueueConnection::connect(&cfg).await;

    let queue = hanekawa_queue::AmqpTaskQueue::new(queue_conn.clone()).await;

    let services = hanekawa_common::Services {
        peer_repository: Arc::new(storage.peer),
        info_hash_repository: Arc::new(storage.info_hash),
        task_queue: Arc::new(queue),
    };

    let hh = tokio::spawn(start_http(cfg.clone(), services.clone(), kt.child_token()));
    let uh = tokio::spawn(start_udp(cfg.clone(), services.clone(), kt.child_token()));

    let background_tasks =
        hanekawa_queue::BackgroundTaskService::new(queue_conn.clone(), services.clone()).await;

    let tkt = kt.child_token();
    let bt = tokio::spawn(async move { background_tasks.run(tkt).await });

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

    let _ = tokio::join!(cancel, hh, uh, bt);
}
