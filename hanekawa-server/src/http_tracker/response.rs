use crate::http::encode::Bencode;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use hanekawa::http_tracker::proto::Error;

use axum::http::StatusCode;
use axum::response::IntoResponse;

pub struct Failure(Error);

impl From<Error> for Failure {
    fn from(value: Error) -> Self {
        Self(value)
    }
}

#[derive(serde::Serialize)]
struct FailureResponse {
    #[serde(rename = "failure reason")]
    reason: String,
}

impl IntoResponse for Failure {
    fn into_response(self) -> axum::response::Response {
        let failure_reason = FailureResponse {
            reason: self.0.to_string(),
        };

        let status_code = match self.0 {
            Error::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InfoHashNotAllowed(_) => StatusCode::FORBIDDEN,
            Error::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, Bencode(failure_reason)).into_response()
    }
}

pub struct OrFailure<T>(pub T);

#[async_trait::async_trait]
impl<S: Sync, T> FromRequestParts<S> for OrFailure<T>
where
    T: FromRequestParts<S> + Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, s: &S) -> Result<Self, Self::Rejection> {
        use axum::body::HttpBody;

        let result = T::from_request_parts(parts, s).await.map_err(|resp| {
            let resp = resp.into_response();
            let sc = resp.status();
            let body = resp.into_body().map_data(|bs| {
                let body = String::from_utf8_lossy(&bs).to_string();
                let failure = FailureResponse { reason: body };
                hanekawa_bencode::to_bytes(&failure).unwrap()
            });

            (sc, body).into_response()
        })?;

        Ok(Self(result))
    }
}
