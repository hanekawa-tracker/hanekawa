use crate::http::encode::Bencode;
use hanekawa::http_tracker::proto::Error;

use axum::http::StatusCode;
use axum::response::IntoResponse;

pub struct Failure(Error);

impl From<Error> for Failure {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> axum::response::Response {
        #[derive(serde::Serialize)]
        struct FailureResponse {
            #[serde(rename = "failure reason")]
            reason: String,
        }

        let failure_reason = FailureResponse {
            reason: self.0.to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, Bencode(failure_reason)).into_response()
    }
}
