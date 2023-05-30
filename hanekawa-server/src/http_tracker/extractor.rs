use axum::{extract::FromRequestParts, http::request::Parts};

pub struct Query<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
T: serde::de::DeserializeOwned,
S: Send + Sync {
    type Rejection = axum::extract::rejection::QueryRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query_string = parts.uri.query().unwrap_or_default();
        let value = hanekawa_percent_encode::from_query_string(query_string);
        
        Ok(Query(value))
    }
}
