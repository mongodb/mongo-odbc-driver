use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    MongoError(#[from] mongodb::error::Error),
}

impl Error {
    pub fn code(&self) -> i32 {
        // using `match` instead of `if let` in case we add future variants
        match self {
            Error::MongoError(m) => {
                match m.kind.as_ref() {
                    ErrorKind::Command(command_error) => command_error.code,
                    // errors other than command errors probably will not concern us, but
                    // the following is included for completeness.
                    ErrorKind::BulkWrite(BulkWriteFailure {
                        write_concern_error: Some(wc_error),
                        ..
                    }) => wc_error.code,
                    ErrorKind::Write(WriteFailure::WriteConcernError(wc_error)) => wc_error.code,
                    _ => 0,
                }
            }
        }
    }
}
