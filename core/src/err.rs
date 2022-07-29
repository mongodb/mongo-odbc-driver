use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    UriFormatError(&'static str),
    #[error(transparent)]
    MongoDriver(#[from] mongodb::error::Error), // Source and Display delegate to mongodb::Error
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::UriFormatError(s) => write!(f, "Uri Format Error: {}", s),
            Error::MongoDriver(mde) => write!(f, "MongoDriverError: {}", mde),
        }
    }
}
