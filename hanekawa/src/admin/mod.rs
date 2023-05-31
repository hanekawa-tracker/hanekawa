use std::sync::Arc;

use hanekawa_common::{
    repository::info_hash::{InfoHashRepository, UpdateInfoHash},
    types::{InfoHash, InfoHashStatus},
    Config,
};

#[derive(Debug)]
pub enum Error {
    NotAllowed,
}

#[derive(Clone)]
pub struct AdminService {
    config: Config,
    info_hash_repository: Arc<dyn InfoHashRepository>,
}

pub struct KnownInfoHashRequest {
    pub hex_info_hash: String,
    pub action: InfoHashStatus,
}

impl AdminService {
    pub fn new(config: &Config, info_hash_repository: Arc<dyn InfoHashRepository>) -> Self {
        let config = config.clone();

        Self {
            config,
            info_hash_repository,
        }
    }

    pub async fn known_info_hash_command(
        &self,
        command: KnownInfoHashRequest,
    ) -> Result<(), Error> {
        if !self.config.enable_admin_api {
            return Err(Error::NotAllowed);
        }

        let info_hash = InfoHash::from_hex(command.hex_info_hash);

        self.info_hash_repository
            .update_info_hash(UpdateInfoHash {
                info_hash: &info_hash,
                status: command.action,
            })
            .await
            .unwrap();

        Ok(())
    }
}
