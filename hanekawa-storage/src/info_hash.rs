use hanekawa_common::repository::{
    info_hash::{GetInfoHashSummary, InfoHashRepository as Repository, UpdateInfoHash},
    Error,
};
use hanekawa_common::types::{InfoHashStatus, InfoHashSummary};

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
    async fn get_info_hash_summary(
        &self,
        cmd: GetInfoHashSummary<'_>,
    ) -> Result<InfoHashSummary, Error> {
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

    async fn update_info_hash(&self, cmd: UpdateInfoHash<'_>) -> Result<(), Error> {
        if let InfoHashStatus::Unknown = cmd.status {
            sqlx::query!(
                "
DELETE FROM info_hashes
WHERE info_hash = $1
",
                &cmd.info_hash.0
            )
            .execute(&self.pool)
            .await
            .unwrap();
        } else {
            let is_allowed = cmd.status == InfoHashStatus::ExplicitAllow;

            sqlx::query!(
                "
INSERT INTO info_hashes(info_hash, is_allowed)
VALUES($1, $2)
ON CONFLICT (info_hash) DO UPDATE
SET is_allowed = $2
",
                &cmd.info_hash.0,
                is_allowed
            )
            .execute(&self.pool)
            .await
            .unwrap();
        }

        Ok(())
    }
}
