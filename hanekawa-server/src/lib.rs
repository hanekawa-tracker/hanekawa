mod http_tracker;
mod udp_tracker;

use http_tracker::tracker;

use axum::Router;

async fn start_http() {
    let tracker = tracker().await;
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

    let hh = tokio::spawn(start_http());
    let uh = tokio::spawn(start_udp());

    let _ = tokio::join!(hh, uh);
}
