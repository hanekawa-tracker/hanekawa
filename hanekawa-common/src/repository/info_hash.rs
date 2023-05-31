use crate::types::{InfoHash, InfoHashSummary};

use super::Error;

#[derive(Debug, Clone)]
pub struct GetInfoHashSummary<'a> {
    pub info_hash: &'a InfoHash,
}

#[async_trait::async_trait]
pub trait InfoHashRepository: Send + Sync {
    async fn get_info_hash_summary(
        &self,
        cmd: GetInfoHashSummary<'_>,
    ) -> Result<InfoHashSummary, Error>;
}
