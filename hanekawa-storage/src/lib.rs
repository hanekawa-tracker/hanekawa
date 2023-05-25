use hanekawa_common::{
    types::{Event, Peer, PeerScrapeData},
    Config,
};

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::types::time::OffsetDateTime;
use std::net::IpAddr;

pub struct UpdatePeerAnnounceCommand {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: IpAddr,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: Option<Event>,
    pub last_update_ts: OffsetDateTime,
}

pub struct ScrapeQuery {
    pub info_hashes: Vec<String>,
}

#[derive(Clone)]
pub struct PeerRepository {
    pool: PgPool,
    cfg: Config,
}

impl PeerRepository {
    pub async fn new(cfg: &Config) -> Option<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(80)
            .connect(&cfg.database_url)
            .await
            .ok()?;

        let cfg = cfg.clone();

        Some(Self { pool, cfg })
    }

    pub async fn update_peer_announce(&self, cmd: &UpdatePeerAnnounceCommand) -> Vec<Peer> {
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
            &cmd.info_hash,
            &cmd.peer_id,
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

        let active_peer_window_start = OffsetDateTime::now_utc()
            - std::time::Duration::from_secs(self.cfg.peer_activity_timeout as u64);

        let peers = sqlx::query!(
            "
SELECT peer_id, ip, port
FROM peer_announces
WHERE
  info_hash = $1 AND ip <> $2
  AND last_update_ts > $3
",
            &cmd.info_hash,
            &inet,
            active_peer_window_start
        )
        .map(|r| Peer {
            peer_id: r.peer_id,
            ip: r.ip.ip(),
            port: r.port as u16,
        })
        .fetch_all(&self.pool)
        .await
        .unwrap();

        peers
    }

    pub async fn scrape(&self, cmd: &ScrapeQuery) -> Vec<(String, PeerScrapeData)> {
        // TODO: active peer distinction for 'downloading'
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
            &cmd.info_hashes
        )
        .map(|r| {
            (
                r.info_hash,
                PeerScrapeData {
                    complete: r.complete.unwrap_or(0) as u32,
                    downloaded: r.complete.unwrap_or(0) as u32,
                    incomplete: r.incomplete.unwrap_or(0) as u32,
                },
            )
        })
        .fetch_all(&self.pool)
        .await
        .unwrap();

        result
    }
}
