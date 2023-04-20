use crate::http_tracker::tracker;

use axum::Router;

pub async fn start() {
    let app = Router::new().nest("/", tracker());

    axum::Server::bind(&([127, 0, 0, 1], 8001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
