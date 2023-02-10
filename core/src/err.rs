use constants::{
    GENERAL_ERROR, INVALID_CURSOR_STATE, INVALID_DESCRIPTOR_INDEX, NO_DSN_OR_DRIVER,
    TIMEOUT_EXPIRED, UNABLE_TO_CONNECT,
};
use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Parsing metadata for {0} gave: {1}")]
    BsonDeserialization(String, bson::de::Error),
    #[error("Column index {0} out of bounds")]
    ColIndexOutOfBounds(u16),
    #[error("Invalid cursor state: cursor not advanced")]
    InvalidCursorState,
    #[error("Result set metadata JSON schema must be object with properties")]
    InvalidResultSetJsonSchema,
    #[error("Invalid Uri: {0}")]
    InvalidUriFormat(String),
    #[error("Field '{0}' schema missing BSON type")]
    MissingFieldBsonType(String),
    #[error("Invalid connection string. Parse error: {0}")]
    MongoParseConnectionString(mongodb::error::Error),
    #[error("Trying to {0} gave: {1}")]
    Mongo(String, mongodb::error::Error),
    #[error("No database provided for query")]
    NoDatabase,
    #[error("Unknown column '{0}' in result set schema")]
    UnknownColumn(String),
    #[error("Trying to access collection '{0}' gave: {1}")]
    ValueAccess(String, bson::document::ValueAccessError),
    #[error("Missing connection {0}")]
    MissingConnection(&'static str),
}

impl Error {
    pub fn get_sql_state(&self) -> &'static str {
        match self {
            Error::Mongo(_, err) => {
                if matches!(err.kind.as_ref(), ErrorKind::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::TimedOut)
                {
                    return TIMEOUT_EXPIRED;
                }
                GENERAL_ERROR
            }
            Error::InvalidUriFormat(_) => UNABLE_TO_CONNECT,
            Error::MongoParseConnectionString(_) => UNABLE_TO_CONNECT,
            Error::NoDatabase => NO_DSN_OR_DRIVER,
            Error::ColIndexOutOfBounds(_) => INVALID_DESCRIPTOR_INDEX,
            Error::InvalidCursorState => INVALID_CURSOR_STATE,
            Error::BsonDeserialization(_, _)
            | Error::UnknownColumn(_)
            | Error::ValueAccess(_, _)
            | Error::InvalidResultSetJsonSchema
            | Error::MissingConnection(_)
            | Error::MissingFieldBsonType(_) => GENERAL_ERROR,
        }
    }

    pub fn code(&self) -> i32 {
        // using `match` instead of `if let` in case we add future variants
        match self {
            Error::Mongo(_, m) | Error::MongoParseConnectionString(m) => {
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
            | Error::InvalidUriFormat(_)
            | Error::InvalidCursorState
            | Error::InvalidResultSetJsonSchema
            | Error::UnknownColumn(_)
            | Error::MissingFieldBsonType(_)
            | Error::ColIndexOutOfBounds(_)
            | Error::BsonDeserialization(_, _)
            | Error::MissingConnection(_)
            | Error::ValueAccess(_, _) => 0,
        }
    }
}
