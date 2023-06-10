use hanekawa_common::Config;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;

pub mod info_hash;
pub mod peer;

pub struct Services {
    pub peer: peer::PeerRepository,
    pub info_hash: info_hash::InfoHashRepository,
}

impl Services {
    pub async fn start(cfg: &Config) -> Self {
        let mut connect_options: PgConnectOptions = cfg.database_url.parse().unwrap();

        connect_options.log_statements(log::LevelFilter::Trace);

        let pool = PgPoolOptions::new()
            .max_connections(80)
            .connect_with(connect_options)
            .await
            .unwrap();

        sqlx::migrate!().run(&pool).await.unwrap();

        let peer = peer::PeerRepository::new(pool.clone(), cfg);
        let info_hash = info_hash::InfoHashRepository::new(pool);

        Self { peer, info_hash }
    }
}
