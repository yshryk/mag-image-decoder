use std::io;
use failure::Fail;

/// Library error
#[derive(Fail, Debug)]
pub enum Error {
    /// Invalid file format
    #[fail(display = "Invalid format: {}", _0)]
    InvalidFormat(String),

    /// Other error
    #[fail(display = "Error: {}", _0)]
    OtherError(String),

    /// Error from io::Error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn other_err(msg: impl Into<String>) -> Error {
    Error::OtherError(msg.into())
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
