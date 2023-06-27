use std::sync::OnceLock;

use crate::Error;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Default, Clone)]
pub struct Config {
    pub chain_id: String,
    pub quorum_type: u8,
}

pub fn set_once(config: Config) -> Result<(), Error> {
    CONFIG.set(config).map_err(|_| Error::AlreadyInitialized)
}

pub fn get_config() -> Result<Config, Error> {
    CONFIG.get().ok_or(Error::NotInitialized).map(|c| c.clone())
}
