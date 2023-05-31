use hanekawa_common::types::{InfoHashStatus, InfoHashSummary};
use hanekawa_common::repository::{Error, info_hash::{InfoHashRepository as Repository, GetInfoHashSummary}};

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct InfoHashRepository {
    pool: PgPool,
}

impl InfoHashRepository {
    pub(super) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Repository for InfoHashRepository {
    async fn get_info_hash_summary(&self, cmd: GetInfoHashSummary<'_>) -> Result<InfoHashSummary, Error> {
        let result = sqlx::query!(
            "
SELECT info_hash, is_allowed
FROM info_hashes
WHERE info_hash = $1
",
            &cmd.info_hash.0
        )
            .map(|r| InfoHashSummary {
                info_hash: cmd.info_hash.clone(),
                status: if r.is_allowed {
                    InfoHashStatus::ExplicitAllow
                } else {
                    InfoHashStatus::ExplicitDeny
                },
            })
            .fetch_optional(&self.pool)
            .await
            .unwrap();

        Ok(result.unwrap_or(InfoHashSummary {
            info_hash: cmd.info_hash.clone(),
            status: InfoHashStatus::Unknown,
        }))
    }
}
