use hanekawa_common::{
    repository::{Error, peer::{GetPeers, UpdatePeerAnnounce, PeerRepository as Repository, GetPeerStatistics}},
    types::{InfoHash, Peer, PeerId, PeerStatistics},
    Config,
};

use sqlx::postgres::PgPool;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::types::time::OffsetDateTime;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PeerRepository {
    pool: PgPool,
    cfg: Config,
}

#[async_trait::async_trait]
impl Repository for PeerRepository {
    async fn update_peer_announce(&self, cmd: UpdatePeerAnnounce<'_>) -> Result<(), Error> {
        let inet: IpNetwork = cmd.ip.clone().into();

        sqlx::query!(
            "
INSERT INTO peer_announces(
  info_hash,
  peer_id,
  ip,
  port,
  uploaded,
  downloaded,
  remaining,
  event,
  last_update_ts
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT (info_hash, peer_id) DO UPDATE
  SET
    ip = $3,
    port = $4,
    uploaded = $5,
    downloaded = $6,
    remaining = $7,
    event = $8,
    last_update_ts = $9;
",
            &cmd.info_hash.0,
            &cmd.peer_id.0,
            &inet,
            cmd.port as i32,
            cmd.uploaded as i64,
            cmd.downloaded as i64,
            cmd.left as i64,
            "started",
            OffsetDateTime::now_utc()
        )
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
    
    async fn get_peers(&self, cmd: GetPeers<'_>) -> Result<Vec<Peer>, Error> {
        let active_peer_window_start = OffsetDateTime::now_utc()
            - std::time::Duration::from_secs(self.cfg.peer_activity_timeout as u64);

        let peers = sqlx::query!(
            "
SELECT peer_id, ip, port
FROM peer_announces
WHERE
  info_hash = $1
  AND last_update_ts > $2
",
            &cmd.info_hash.0,
            active_peer_window_start
        )
            .map(|r| Peer {
                peer_id: PeerId(r.peer_id),
                ip: r.ip.ip(),
                port: r.port as u16,
            })
            .fetch_all(&self.pool)
            .await
            .unwrap();

        Ok(peers)
    }

    async fn get_peer_statistics(&self, cmd: GetPeerStatistics<'_>) -> Result<HashMap<InfoHash, PeerStatistics>, Error> {
        let ih_bs: Vec<Vec<u8>> = cmd.info_hashes.iter().cloned().map(|ih| ih.0).collect();

        let result = sqlx::query!(
            "
SELECT
  info_hash,
  COUNT(*) FILTER (WHERE remaining = 0) AS complete,
  COUNT(*) FILTER (WHERE remaining <> 0) AS incomplete
FROM
  peer_announces
WHERE info_hash = ANY($1)
GROUP BY info_hash
",
            &ih_bs
        )
            .map(|r| {
                (
                InfoHash(r.info_hash),
                PeerStatistics {
                    complete: r.complete.unwrap_or(0) as u32,
                    downloaded: r.complete.unwrap_or(0) as u32,
                    incomplete: r.incomplete.unwrap_or(0) as u32,
                },
                )
            })
            .fetch_all(&self.pool)
            .await
            .unwrap();

        Ok(result.into_iter().collect())
    }
}

impl PeerRepository {
    pub(super) fn new(pool: PgPool, cfg: &Config) -> Self {
        let cfg = cfg.clone();
        Self { pool, cfg }
    }
}
