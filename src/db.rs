use std::sync::OnceLock;
use std::time::Duration;

use crate::config::config;
use crate::error::Result;
use sqlx::{Pool, Postgres, pool::PoolOptions};
use tokio::sync::OnceCell;

pub type Db = Pool<Postgres>;

pub async fn get_or_init_db() -> Result<Db> {
    static INIT: OnceCell<Db> = OnceCell::const_new();

    //panic early if db connection fails
    let db = INIT.get_or_init(async || connect().await.unwrap()).await;

    Ok(db.clone())
}

pub async fn connect() -> Result<Db> {
    let config = config();
    let db: Db = PoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.DB_URL)
        .await?;
    Ok(db)
}
