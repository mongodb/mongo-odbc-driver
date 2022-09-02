use constants::{GENERAL_ERROR, INVALID_CONN_ATTRIB, TIMEOUT_EXPIRED};
use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid connection string. Parse error: {0}")]
    MongoParseError(mongodb::error::Error),
    #[error(transparent)]
    MongoError(#[from] mongodb::error::Error),
}

impl Error {
    pub fn get_sql_state(&self) -> &'static str {
        match self {
            Error::MongoError(err) => {
                if matches!(err.kind.as_ref(), ErrorKind::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::TimedOut)
                {
                    return TIMEOUT_EXPIRED;
                }
                GENERAL_ERROR
            }
            Error::MongoParseError(_) => INVALID_CONN_ATTRIB,
        }
    }

    pub fn code(&self) -> i32 {
        // using `match` instead of `if let` in case we add future variants
        match self {
            Error::MongoError(m) | Error::MongoParseError(m) => {
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
