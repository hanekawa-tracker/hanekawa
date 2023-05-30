use axum::{extract::FromRequestParts, http::request::Parts, response::IntoResponse};

pub struct Query<T>(pub T);

pub struct QueryRejection(serde::de::value::Error);

impl IntoResponse for QueryRejection {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::status::StatusCode::BAD_REQUEST,
            format!("failed to deserialize query string: {}", self.0),
        )
            .into_response()
    }
}

#[async_trait::async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QueryRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query_string = parts.uri.query().unwrap_or_default();
        let value = hanekawa_percent_encode::from_query_string(query_string)
            .map_err(|err| QueryRejection(err))?;

        Ok(Query(value))
    }
}
