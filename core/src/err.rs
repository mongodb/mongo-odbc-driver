use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use thiserror::Error;

// SQL states
pub const HY024: &str = "HY024";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    MongoError(#[from] mongodb::error::Error),
}

impl Error {
    pub fn get_sql_state(&self) -> &'static str {
        match self {
            // TODO: for now we just return HY024 for all Mongo Errors.
            // In the future this will change based on the type of error.
            Error::MongoError(_) => HY024,
        }
    }

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
