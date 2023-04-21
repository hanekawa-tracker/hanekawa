use axum::http::{header, status, HeaderValue};
use axum::response::IntoResponse;

pub struct Bencode<T>(pub T);

const APPLICATION_OCTET_STREAM: &'static str = "application/octet-stream";

impl<T> IntoResponse for Bencode<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let encoded = hanekawa_bencode::to_bytes(&self.0);
        match encoded {
            Ok(bs) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(APPLICATION_OCTET_STREAM),
                )],
                bs,
            )
                .into_response(),
            Err(e) => (
                status::StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"))],
                e.to_string(),
            )
                .into_response(),
        }
    }
}
