use std::{env::var, sync::OnceLock};

use crate::error::{Error, Result};

#[allow(non_snake_case)]
pub struct Config {
    pub RPC_ENDPOINT: String,
}

pub fn config() -> &'static Config {
    static INITIAL: OnceLock<Config> = OnceLock::new();

    let config = INITIAL.get_or_init(|| init_config().unwrap());
    config
}

fn init_config() -> Result<Config> {
    Ok(Config {
        RPC_ENDPOINT: get_env("SERVICE_RPC_ENDPOINT")?,
    })
}

fn get_env(name: &'static str) -> Result<String> {
    var(name).map_err(|_| Error::ConfigEnvMissing(name))
}
