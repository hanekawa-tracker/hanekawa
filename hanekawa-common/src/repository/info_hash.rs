use crate::types::{InfoHash, InfoHashStatus, InfoHashSummary};

use super::Error;

#[derive(Debug, Clone)]
pub struct GetInfoHashSummary<'a> {
    pub info_hash: &'a InfoHash,
}

#[derive(Debug, Clone)]
pub struct UpdateInfoHash<'a> {
    pub info_hash: &'a InfoHash,
    pub status: InfoHashStatus,
}

#[async_trait::async_trait]
pub trait InfoHashRepository: Send + Sync {
    async fn get_info_hash_summary(
        &self,
        cmd: GetInfoHashSummary<'_>,
    ) -> Result<InfoHashSummary, Error>;

    async fn update_info_hash(&self, cmd: UpdateInfoHash<'_>) -> Result<(), Error>;
}
