use constants::{
    GENERAL_ERROR, INVALID_DESCRIPTOR_INDEX, NO_DSN_OR_DRIVER, TIMEOUT_EXPIRED, UNABLE_TO_CONNECT,
};
use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BsonDeserialization(#[from] bson::de::Error),
    #[error("Column index {0} out of bounds")]
    ColIndexOutOfBounds(u16),
    #[error("Result set metadata JSON schema must be object with properties")]
    InvalidResultSetJsonSchema,
    #[error("Invalid connection string. Parse error: {0}")]
    MongoParseConnectionStringError(mongodb::error::Error),
    #[error(transparent)]
    MongoError(#[from] mongodb::error::Error),
    #[error("No database provided for query")]
    NoDatabase,
    #[error(transparent)]
    ValueAccess(bson::document::ValueAccessError),
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
            Error::MongoParseConnectionStringError(_) => UNABLE_TO_CONNECT,
            Error::NoDatabase => NO_DSN_OR_DRIVER,
            Error::ColIndexOutOfBounds(_) => INVALID_DESCRIPTOR_INDEX,
            Error::BsonDeserialization(_)
            | Error::ValueAccess(_)
            | Error::InvalidResultSetJsonSchema => GENERAL_ERROR,
        }
    }

    pub fn code(&self) -> i32 {
        // using `match` instead of `if let` in case we add future variants
        match self {
            Error::MongoError(m) | Error::MongoParseConnectionStringError(m) => {
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
            Error::NoDatabase
            | Error::InvalidResultSetJsonSchema
            | Error::ColIndexOutOfBounds(_)
            | Error::BsonDeserialization(_)
            | Error::ValueAccess(_) => 0,
        }
    }
}
