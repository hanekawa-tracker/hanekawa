use hanekawa_common::Config;
use sqlx::postgres::PgPoolOptions;

pub mod info_hash;
pub mod peer;

pub struct Services {
    pub peer: peer::PeerRepository,
    pub info_hash: info_hash::InfoHashRepository,
}

impl Services {
    pub async fn start(cfg: &Config) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(80)
            .connect(&cfg.database_url)
            .await
            .unwrap();

        let peer = peer::PeerRepository::new(pool.clone(), cfg);
        let info_hash = info_hash::InfoHashRepository::new(pool);

        Self { peer, info_hash }
    }
}
