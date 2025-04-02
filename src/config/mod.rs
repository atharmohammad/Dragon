use std::{env::var, sync::OnceLock};

use crate::error::{Error, Result};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Config {
    pub HELIUS_API_KEY: String,
    pub WEB_FOLDER: String,
    pub WEBHOOK_ID: Option<String>,
    pub DB_URL: String,
}

pub fn config() -> &'static Config {
    static INITIAL: OnceLock<Config> = OnceLock::new();

    let config = INITIAL.get_or_init(|| init_config().unwrap());
    config
}

fn init_config() -> Result<Config> {
    Ok(Config {
        HELIUS_API_KEY: get_env("SERVICE_HELIUS_API_KEY")?,
        WEB_FOLDER: String::from("public"),
        WEBHOOK_ID: get_optional_env("SERVICE_WEBHOOK_ID"),
        DB_URL: get_env("SERVICE_DB_URL")?,
    })
}

fn get_env(name: &'static str) -> Result<String> {
    var(name).map_err(|_| Error::ConfigEnvMissing(name))
}

fn get_optional_env(name: &str) -> Option<String> {
    var(name).ok()
}
