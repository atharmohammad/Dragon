use core::fmt;

use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[allow(non_snake_case)]
#[derive(Debug, From)]
pub enum Error {
    WebhookIdMissing,

    // -- Helius Error
    #[from]
    Helius(helius::error::HeliusError),

    // -- Config Errors
    ConfigEnvMissing(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for Error {}
