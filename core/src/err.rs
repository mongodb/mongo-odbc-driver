use constants::{
    OdbcState, FUNCTION_SEQUENCE_ERROR, GENERAL_ERROR, INVALID_CURSOR_STATE,
    INVALID_DESCRIPTOR_INDEX, NO_DSN_OR_DRIVER, OPERATION_CANCELLED, TIMEOUT_EXPIRED,
    UNABLE_TO_CONNECT,
};
use mongodb::error::{ErrorKind, WriteFailure};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Column index {0} out of bounds")]
    ColIndexOutOfBounds(u16),
    #[error("Trying to access collection metadata failed with: {0}")]
    CollectionCursorUpdate(mongodb::error::Error),
    #[error("Getting metadata for collection '{0}' failed with error: {1}")]
    CollectionDeserialization(String, mongodb::bson::de::Error),
    #[error("Retrieving information for database failed with error: {0}")]
    DatabaseVersionRetreival(mongodb::error::Error),
    #[error("Getting database metadata failed with error: {0}")]
    DatabaseVersionDeserialization(mongodb::bson::de::Error),
    #[error("Setting connection options failed with error: {0}")]
    InvalidClientOptions(mongodb::error::Error),
    #[error("Invalid cursor state: cursor not advanced")]
    InvalidCursorState,
    #[error("{0}")]
    InvalidResultSetJsonSchema(&'static str),
    #[error("Invalid Uri: {0}")]
    InvalidUriFormat(String),
    #[error("Field '{0}' schema missing BSON type")]
    MissingFieldBsonType(String),
    #[error("Invalid connection string. Parse error: {0}")]
    MongoParseConnectionString(mongodb::error::Error),
    #[error("No database provided for query")]
    NoDatabase,
    #[error("Query was cancelled")]
    QueryCancelled,
    #[error("Getting query result failed with error: {0}")]
    QueryCursorUpdate(mongodb::error::Error),
    #[error("Getting metadata for query failed with error: {0}")]
    QueryDeserialization(mongodb::bson::de::Error),
    #[error("Trying to execute query failed with error: {0:?}")]
    QueryExecutionFailed(mongodb::error::Error),
    #[error("Unknown column '{0}' in result set schema")]
    UnknownColumn(String),
    #[error("Error retrieving data for field {0}: {1}")]
    ValueAccess(String, mongodb::bson::document::ValueAccessError),
    #[error("Missing connection {0}")]
    MissingConnection(&'static str),
    #[error("Unsupported cluster configuration: {0}")]
    UnsupportedClusterConfiguration(String),
    #[error("Unsupported operation {0}")]
    UnsupportedOperation(&'static str),
    #[error("Statement not executed")]
    StatementNotExecuted,
    #[error("The driver version `{0}` is incompatible with libmongosqltranslate")]
    LibmongosqltranslateLibraryIsIncompatible(&'static str),
    #[error("The schema document for collection `{0}` could not be found in the `__sql_schemas_` collection")]
    SchemaDocumentNotFoundInSchemaCollection(String),
    #[error(
        "The libmongosqltranslate command `{0}` failed. Error message: `{1}`. Error is internal: {2}"
    )]
    LibmongosqltranslateCommandError(String, String, bool),
    #[error("Loading the runCommand symbol from libmongosqltranslate failed with error: {0}")]
    RunCommandSymbolNotFound(libloading::Error),
    #[error("Deserializing libmongosqltranslate response failed with error: {0}")]
    LibmongosqltranslateDeserializationError(mongodb::bson::de::Error),
    #[error("Serializing Command Document for libmongosqltranslate failed with error: {0}")]
    LibmongosqltranslateSerializationError(mongodb::bson::ser::Error),
}

impl Error {
    pub fn get_sql_state(&self) -> OdbcState {
        match self {
            Error::CollectionCursorUpdate(err)
            | Error::DatabaseVersionRetreival(err)
            | Error::InvalidClientOptions(err)
            | Error::QueryCursorUpdate(err)
            | Error::QueryExecutionFailed(err) => {
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
            Error::CollectionDeserialization(_, _)
            | Error::DatabaseVersionDeserialization(_)
            | Error::InvalidResultSetJsonSchema(_)
            | Error::MissingConnection(_)
            | Error::MissingFieldBsonType(_)
            | Error::QueryDeserialization(_)
            | Error::UnknownColumn(_)
            | Error::ValueAccess(_, _)
            | Error::UnsupportedClusterConfiguration(_)
            | Error::UnsupportedOperation(_)
            | Error::LibmongosqltranslateLibraryIsIncompatible(_)
            | Error::SchemaDocumentNotFoundInSchemaCollection(_)
            | Error::LibmongosqltranslateCommandError(_, _, _)
            | Error::RunCommandSymbolNotFound(_)
            | Error::LibmongosqltranslateDeserializationError(_)
            | Error::LibmongosqltranslateSerializationError(_) => GENERAL_ERROR,
            Error::StatementNotExecuted => FUNCTION_SEQUENCE_ERROR,
            Error::QueryCancelled => OPERATION_CANCELLED,
        }
    }

    pub fn code(&self) -> i32 {
        // using `match` instead of `if let` in case we add future variants
        match self {
            Error::CollectionCursorUpdate(m)
            | Error::DatabaseVersionRetreival(m)
            | Error::InvalidClientOptions(m)
            | Error::QueryCursorUpdate(m)
            | Error::QueryExecutionFailed(m)
            | Error::MongoParseConnectionString(m) => match m.kind.as_ref() {
                ErrorKind::Command(command_error) => command_error.code,
                ErrorKind::Write(WriteFailure::WriteConcernError(wc_error)) => wc_error.code,
                ErrorKind::BulkWrite(bulk_error) => bulk_error
                    .write_errors
                    .iter()
                    // invoking the axiom of choice here ;)
                    .last()
                    .map_or(0, |(_, e)| e.code),
                _ => 0,
            },
            Error::ColIndexOutOfBounds(_)
            | Error::CollectionDeserialization(_, _)
            | Error::DatabaseVersionDeserialization(_)
            | Error::InvalidCursorState
            | Error::InvalidResultSetJsonSchema(_)
            | Error::InvalidUriFormat(_)
            | Error::MissingConnection(_)
            | Error::MissingFieldBsonType(_)
            | Error::NoDatabase
            | Error::QueryCancelled
            | Error::QueryDeserialization(_)
            | Error::UnknownColumn(_)
            | Error::ValueAccess(_, _)
            | Error::UnsupportedOperation(_)
            | Error::UnsupportedClusterConfiguration(_)
            | Error::StatementNotExecuted
            | Error::LibmongosqltranslateLibraryIsIncompatible(_)
            | Error::SchemaDocumentNotFoundInSchemaCollection(_)
            | Error::LibmongosqltranslateCommandError(_, _, _)
            | Error::RunCommandSymbolNotFound(_)
            | Error::LibmongosqltranslateDeserializationError(_)
            | Error::LibmongosqltranslateSerializationError(_) => 0,
        }
    }
}
