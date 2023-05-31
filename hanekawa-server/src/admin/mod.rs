use hanekawa::admin::{AdminService, Error, KnownInfoHashRequest};
use hanekawa_common::Config;

use crate::http::extractor::Query;

use axum::routing::{delete, post};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use axum::{response::IntoResponse, Router};
use hanekawa_common::types::InfoHashStatus;

#[derive(Debug, serde::Deserialize)]
struct UpdateParams {
    allowed: bool,
}

async fn delete_info_hash(
    Path(hex_info_hash): Path<String>,
    State(admin): State<AdminService>,
) -> impl IntoResponse {
    let result = admin
        .known_info_hash_command(KnownInfoHashRequest {
            hex_info_hash,
            action: InfoHashStatus::Unknown,
        })
        .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(Error::NotAllowed) => StatusCode::NOT_FOUND,
    }
}

async fn update_info_hash(
    Path(hex_info_hash): Path<String>,
    Query(params): Query<UpdateParams>,
    State(admin): State<AdminService>,
) -> impl IntoResponse {
    let result = admin
        .known_info_hash_command(KnownInfoHashRequest {
            hex_info_hash,
            action: if params.allowed {
                InfoHashStatus::ExplicitAllow
            } else {
                InfoHashStatus::ExplicitDeny
            },
        })
        .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(Error::NotAllowed) => StatusCode::NOT_FOUND,
    }
}

pub async fn admin<S>(cfg: &Config) -> Router<S> {
    let storage = hanekawa_storage::Services::start(cfg).await;

    let ir = std::sync::Arc::new(storage.info_hash);
    let admin = AdminService::new(cfg, ir);

    Router::new()
        .route("/info_hashes/:info_hash", delete(delete_info_hash))
        .route("/info_hashes/:info_hash", post(update_info_hash))
        .with_state(admin)
}
