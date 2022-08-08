use mongodb::error::{BulkWriteFailure, ErrorKind, WriteFailure};
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    UriFormatError(String),
    MongoError(MongoError),
}

impl Error {
    pub fn code(&self) -> i32 {
        match self {
            Error::UriFormatError(_) => 0,
            Error::MongoError(m) => m.code.unwrap_or(0),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::UriFormatError(s) => write!(f, "Uri Format Error: {}", s),
            Error::MongoError(mde) => write!(f, "Error returned from MongoDB: {:?}", mde),
        }
    }
}

#[derive(Debug)]
pub struct MongoError {
    message: String,
    code: Option<i32>,
}

impl From<mongodb::error::Error> for Error {
    fn from(me: mongodb::error::Error) -> Error {
        Error::MongoError(MongoError {
            message: format!("{:?}", me),
            code: match me.kind.as_ref() {
                ErrorKind::Command(command_error) => Some(command_error.code),
                // errors other than command errors probably will not concern us, but
                // the following is included for completeness.
                ErrorKind::BulkWrite(BulkWriteFailure {
                    write_concern_error: Some(wc_error),
                    ..
                }) => Some(wc_error.code),
                ErrorKind::Write(WriteFailure::WriteConcernError(wc_error)) => Some(wc_error.code),
                _ => None,
            },
        })
    }
}

impl Display for MongoError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.code {
            Some(c) => write!(
                f,
                "MongoDB returned error code {} due to: {}",
                c, self.message
            ),
            None => write!(f, "MongoDB returned {}", self.message),
        }
    }
}
