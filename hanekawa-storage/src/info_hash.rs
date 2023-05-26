use hanekawa_common::types::{InfoHash, InfoHashStatus, InfoHashSummary};

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

pub struct InfoHashSummaryQuery<'a> {
    pub info_hash: &'a InfoHash,
}

impl InfoHashRepository {
    pub async fn get_summary(&self, cmd: InfoHashSummaryQuery<'_>) -> InfoHashSummary {
        let result = sqlx::query!(
            "
SELECT info_hash, is_allowed FROM info_hashes
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

        result.unwrap_or(InfoHashSummary {
            info_hash: cmd.info_hash.clone(),
            status: InfoHashStatus::Unknown,
        })
    }
}
