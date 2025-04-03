use core::fmt;
use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse};
use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[allow(non_snake_case)]
#[derive(Debug, From)]
pub enum Error {
    WebhookIdMissing,
    FailedToIndexInBlockTxMapVector,
    FailedToParseInput,
    // -- Helius Error
    #[from]
    Helius(helius::error::HeliusError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for Error {}
