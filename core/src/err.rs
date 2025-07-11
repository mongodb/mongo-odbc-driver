use constants::{
    OdbcState, FUNCTION_SEQUENCE_ERROR, GENERAL_ERROR, INVALID_CURSOR_STATE,
    INVALID_DESCRIPTOR_INDEX, NO_DSN_OR_DRIVER, OPERATION_CANCELLED, TIMEOUT_EXPIRED,
    UNABLE_TO_CONNECT,
};
use mongodb::error::{ErrorKind, WriteFailure};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct QueryDiagnostics {
    pub query: String,
    pub schema: String,
    pub translation: String,
}

impl QueryDiagnostics {
    pub fn new(query: String, schema: String, translation: String) -> Self {
        Self {
            query,
            schema,
            translation,
        }
    }
}

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
    #[error("Error retrieving metadata for field {0}.{1}")]
    MetadataAccess(String, String),
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
    #[error("The mongosql translate command `{0}` failed. Error message: `{1}`.")]
    MongoSQLCommandFailed(&'static str, String),
    #[error("The client app_name is empty. However, this shouldn't be possible.")]
    EmptyAppName,
    #[error(
        "The mongosql Translation `pipeline` should be an array; however this was not the case."
    )]
    TranslationPipelineNotArray,
    #[error("The mongosql Translation `pipeline` array should only contain Documents; however, a non-document bson-type was encountered.")]
    TranslationPipelineArrayContainsNonDocument,
    #[error(
        "Multiple Documents were returned when getting the schema; however, only one was expected."
    )]
    MultipleSchemaDocumentsReturned(usize),
    #[error("The buildInfo command failed with the following error: `{0}`")]
    BuildInfoCmdExecutionFailed(mongodb::error::Error),
    #[error("Library path error: {0}")]
    LibraryPathError(String),
}

impl Error {
    pub fn get_sql_state(&self) -> OdbcState<'_> {
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
            | Error::MongoSQLCommandFailed(_, _)
            | Error::EmptyAppName
            | Error::TranslationPipelineNotArray
            | Error::TranslationPipelineArrayContainsNonDocument
            | Error::MultipleSchemaDocumentsReturned(_)
            | Error::MetadataAccess(_, _)
            | Error::BuildInfoCmdExecutionFailed(_) => GENERAL_ERROR,
            Error::StatementNotExecuted => FUNCTION_SEQUENCE_ERROR,
            Error::QueryCancelled => OPERATION_CANCELLED,
            Error::LibraryPathError(_) => GENERAL_ERROR,
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
            | Error::MongoSQLCommandFailed(_, _)
            | Error::EmptyAppName
            | Error::TranslationPipelineNotArray
            | Error::TranslationPipelineArrayContainsNonDocument
            | Error::LibraryPathError(_)
            | Error::MultipleSchemaDocumentsReturned(_)
            | Error::BuildInfoCmdExecutionFailed(_)
            | Error::MetadataAccess(_, _) => 0,
        }
    }
}
