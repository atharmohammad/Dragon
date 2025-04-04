use core::fmt;
use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse};
use derive_more::From;

use crate::indexer;

pub type Result<T> = core::result::Result<T, Error>;

#[allow(non_snake_case)]
#[derive(Debug, From)]
pub enum Error {
    #[from]
    Mutex(String),

    // -- Helius Error
    #[from]
    Indexer(indexer::error::Error),

    // -- Helius Error
    #[from]
    Sqlx(sqlx::error::Error),

    // -- Config Errors
    ConfigEnvMissing(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        dbg!("ERROR: {self:}");

        let mut response = StatusCode::BAD_REQUEST.into_response();
        response.extensions_mut().insert(Arc::new(self));

        response
    }
}
