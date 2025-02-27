use constants::{
    OdbcState, CONNECTION_NOT_OPEN, FETCH_TYPE_OUT_OF_RANGE, FRACTIONAL_TRUNCATION, GENERAL_ERROR,
    GENERAL_WARNING, INDICATOR_VARIABLE_REQUIRED, INTEGRAL_TRUNCATION,
    INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER, INVALID_ATTR_VALUE, INVALID_CHARACTER_VALUE,
    INVALID_COLUMN_NUMBER, INVALID_CURSOR_STATE, INVALID_DATETIME_FORMAT, INVALID_DESCRIPTOR_INDEX,
    INVALID_DRIVER_COMPLETION, INVALID_FIELD_DESCRIPTOR, INVALID_INFO_TYPE_VALUE, INVALID_SQL_TYPE,
    NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, NO_RESULTSET, OPTION_CHANGED, PROGRAM_TYPE_OUT_OF_RANGE,
    RESTRICTED_DATATYPE, RIGHT_TRUNCATED, VENDOR_IDENTIFIER,
};
use mongo_odbc_core::ErrorDetails;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
#[repr(C)]
pub enum ODBCError {
    #[error("[{vendor}][API] {0}", vendor = VENDOR_IDENTIFIER)]
    General(&'static str),
    #[error("[{vendor}][API] {0}", vendor = VENDOR_IDENTIFIER)]
    GeneralWarning(String),
    #[error("[{vendor}][API] Caught panic: {0}", vendor = VENDOR_IDENTIFIER)]
    Panic(String),
    #[error("[{vendor}][API] The feature {0} is not implemented", vendor = VENDOR_IDENTIFIER)]
    Unimplemented(&'static str),
    #[error("[{vendor}][API] The data type {0} is not implemented", vendor = VENDOR_IDENTIFIER)]
    UnimplementedDataType(String),
    #[error(
        "[{vendor}][API] The driver connect option {0} is not supported",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedDriverConnectOption(String),
    #[error(
        "[{vendor}][API] The connection attribute {0} is not supported",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedConnectionAttribute(String),
    #[error(
        "[{vendor}][API] The statement attribute {0} is not supported",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedStatementAttribute(String),
    #[error(
        "[{vendor}][API] A schema pattern was specified, and the driver does not support schemas",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedFieldSchema(),
    #[error(
        "[{vendor}][API] The field descriptor value {0} is not supported",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedFieldDescriptor(u16),
    #[error(
        "[{vendor}][API] Retrieving value for infoType {0} is not implemented yet",
        vendor = VENDOR_IDENTIFIER
    )]
    UnsupportedInfoTypeRetrieval(String),
    #[error("[{vendor}][API] InfoType {0} out of range", vendor = VENDOR_IDENTIFIER)]
    UnknownInfoType(String),
    #[error(
        "[{vendor}][API] Indicator variable was null when null data was accessed",
        vendor = VENDOR_IDENTIFIER
    )]
    IndicatorVariableRequiredButNotSupplied,
    #[error("[{vendor}][API] The field index {0} is out of bounds", vendor = VENDOR_IDENTIFIER)]
    InvalidDescriptorIndex(u16),
    #[error("[{vendor}][API] The column index {0} is out of bounds", vendor = VENDOR_IDENTIFIER)]
    InvalidColumnNumber(u16),
    #[error("[{vendor}][API] No ResultSet", vendor = VENDOR_IDENTIFIER)]
    InvalidCursorState,
    #[error("[{vendor}][API] Invalid SQL Type: {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidSqlType(String),
    #[error("[{vendor}][API] Invalid handle type, expected {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidHandleType(&'static str),
    #[error("[{vendor}][API] Invalid value for attribute {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidAttrValue(&'static str),
    #[error("[{vendor}][API] Invalid attribute identifier {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidAttrIdentifier(i32),
    #[error("[{vendor}][API] Fetch type out of range {0}", vendor = VENDOR_IDENTIFIER)]
    FetchTypeOutOfRange(i16),
    #[error("[{vendor}][API] Invalid target type {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidTargetType(i16),
    #[error("[{vendor}][API] Invalid driver completion type {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidDriverCompletion(u16),
    #[error(
        "[{vendor}][API] Missing property \"Driver\" or \"DSN\" in connection string",
        vendor = VENDOR_IDENTIFIER
    )]
    MissingDriverOrDSNProperty,
    #[error(
        "[{vendor}][API] Buffer size \"{0}\" not large enough for data",
        vendor = VENDOR_IDENTIFIER
    )]
    OutStringTruncated(usize),
    #[error(
        "[{vendor}][API] floating point data \"{0}\" was truncated to fixed point",
        vendor = VENDOR_IDENTIFIER
    )]
    FractionalTruncation(String),
    #[error(
        "[{vendor}][API] fractional seconds for data \"{0}\" was truncated to nanoseconds",
        vendor = VENDOR_IDENTIFIER
    )]
    FractionalSecondsTruncation(String),
    #[error(
        "[{vendor}][API] fractional seconds data truncated from \"{0}\"",
        vendor = VENDOR_IDENTIFIER
    )]
    SecondsTruncation(String),
    #[error(
        "[{vendor}][API] datetime data \"{0}\" was truncated to date",
        vendor = VENDOR_IDENTIFIER
    )]
    TimeTruncation(String),
    #[error(
        "[{vendor}][API] integral data \"{0}\" was truncated due to overflow",
        vendor = VENDOR_IDENTIFIER
    )]
    IntegralTruncation(String),
    #[error("[{vendor}][API] invalid datetime format", vendor = VENDOR_IDENTIFIER)]
    InvalidDatetimeFormat,
    #[error(
        "[{vendor}][API] invalid character value for cast to type: {0}",
        vendor = VENDOR_IDENTIFIER
    )]
    InvalidCharacterValue(&'static str),
    #[error("[{vendor}][API] Invalid field descriptor value {0}", vendor = VENDOR_IDENTIFIER)]
    InvalidFieldDescriptor(u16),
    #[error(
        "[{vendor}][API] Invalid value for attribute {0}, changed to {1}",
        vendor = VENDOR_IDENTIFIER
    )]
    OptionValueChanged(&'static str, &'static str),
    #[error(
        "[{vendor}][API] BSON type {0} cannot be converted to ODBC type {1}",
        vendor = VENDOR_IDENTIFIER
    )]
    RestrictedDataType(&'static str, &'static str),
    #[error("[{vendor}][API] No resultset for statement", vendor = VENDOR_IDENTIFIER)]
    NoResultSet,
    #[error("Connection not open")]
    ConnectionNotOpen,
    #[error("[{vendor}][Core] {0}", vendor = VENDOR_IDENTIFIER)]
    Core(mongo_odbc_core::Error),
}

pub type Result<T> = std::result::Result<T, ODBCError>;

impl ODBCError {
    pub fn get_sql_state(&self) -> OdbcState {
        match self {
            ODBCError::Unimplemented(_)
            | ODBCError::UnimplementedDataType(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::UnsupportedFieldSchema()
            | ODBCError::UnsupportedConnectionAttribute(_)
            | ODBCError::UnsupportedStatementAttribute(_)
            | ODBCError::UnsupportedInfoTypeRetrieval(_)
            | ODBCError::UnsupportedFieldDescriptor(_) => NOT_IMPLEMENTED,
            ODBCError::General(_) | ODBCError::Panic(_) => GENERAL_ERROR,
            ODBCError::GeneralWarning(_) => GENERAL_WARNING,
            ODBCError::Core(c) => c.get_sql_state(),
            ODBCError::InvalidAttrValue(_) => INVALID_ATTR_VALUE,
            ODBCError::InvalidAttrIdentifier(_) => INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER,
            ODBCError::FetchTypeOutOfRange(_) => FETCH_TYPE_OUT_OF_RANGE,
            ODBCError::InvalidCursorState => INVALID_CURSOR_STATE,
            ODBCError::InvalidHandleType(_) => NOT_IMPLEMENTED,
            ODBCError::InvalidTargetType(_) => PROGRAM_TYPE_OUT_OF_RANGE,
            ODBCError::InvalidDriverCompletion(_) => INVALID_DRIVER_COMPLETION,
            ODBCError::OptionValueChanged(_, _) => OPTION_CHANGED,
            ODBCError::OutStringTruncated(_) => RIGHT_TRUNCATED,
            ODBCError::MissingDriverOrDSNProperty => NO_DSN_OR_DRIVER,
            ODBCError::InvalidDescriptorIndex(_) => INVALID_DESCRIPTOR_INDEX,
            ODBCError::InvalidColumnNumber(_) => INVALID_COLUMN_NUMBER,
            ODBCError::InvalidSqlType(_) => INVALID_SQL_TYPE,
            ODBCError::InvalidFieldDescriptor(_) => INVALID_FIELD_DESCRIPTOR,
            ODBCError::RestrictedDataType(_, _) => RESTRICTED_DATATYPE,
            ODBCError::FractionalTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::FractionalSecondsTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::SecondsTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::TimeTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::IntegralTruncation(_) => INTEGRAL_TRUNCATION,
            ODBCError::InvalidDatetimeFormat => INVALID_DATETIME_FORMAT,
            ODBCError::InvalidCharacterValue(_) => INVALID_CHARACTER_VALUE,
            ODBCError::IndicatorVariableRequiredButNotSupplied => INDICATOR_VARIABLE_REQUIRED,
            ODBCError::NoResultSet => NO_RESULTSET,
            ODBCError::UnknownInfoType(_) => INVALID_INFO_TYPE_VALUE,
            ODBCError::ConnectionNotOpen => CONNECTION_NOT_OPEN,
        }
    }

    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Functions that return these errors don't interact with MongoDB,
            // and so the driver returns 0 since it doesn't have a native error
            // code to propagate.
            ODBCError::Unimplemented(_)
            | ODBCError::General(_)
            | ODBCError::GeneralWarning(_)
            | ODBCError::Panic(_)
            | ODBCError::UnimplementedDataType(_)
            | ODBCError::InvalidAttrValue(_)
            | ODBCError::InvalidAttrIdentifier(_)
            | ODBCError::FetchTypeOutOfRange(_)
            | ODBCError::InvalidCursorState
            | ODBCError::InvalidHandleType(_)
            | ODBCError::InvalidTargetType(_)
            | ODBCError::MissingDriverOrDSNProperty
            | ODBCError::OutStringTruncated(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::UnsupportedConnectionAttribute(_)
            | ODBCError::UnsupportedStatementAttribute(_)
            | ODBCError::UnsupportedFieldSchema()
            | ODBCError::InvalidFieldDescriptor(_)
            | ODBCError::OptionValueChanged(_, _)
            | ODBCError::InvalidDescriptorIndex(_)
            | ODBCError::InvalidColumnNumber(_)
            | ODBCError::RestrictedDataType(_, _)
            | ODBCError::IndicatorVariableRequiredButNotSupplied
            | ODBCError::FractionalTruncation(_)
            | ODBCError::FractionalSecondsTruncation(_)
            | ODBCError::SecondsTruncation(_)
            | ODBCError::TimeTruncation(_)
            | ODBCError::IntegralTruncation(_)
            | ODBCError::InvalidDatetimeFormat
            | ODBCError::InvalidSqlType(_)
            | ODBCError::UnsupportedFieldDescriptor(_)
            | ODBCError::InvalidCharacterValue(_)
            | ODBCError::InvalidDriverCompletion(_)
            | ODBCError::NoResultSet
            | ODBCError::UnsupportedInfoTypeRetrieval(_)
            | ODBCError::ConnectionNotOpen
            | ODBCError::UnknownInfoType(_) => 0,
            ODBCError::Core(me) => me.code(),
        }
    }
}

impl From<mongo_odbc_core::Error> for ODBCError {
    fn from(err: mongo_odbc_core::Error) -> Self {
        if let Some(details) = err.details() {
            log::error!("{details:?}");
        }
        match err {
            mongo_odbc_core::Error::ColIndexOutOfBounds(u) => ODBCError::InvalidDescriptorIndex(u),
            e => ODBCError::Core(e),
        }
    }
}

impl From<&mongo_odbc_core::Error> for ODBCError {
    fn from(err: &mongo_odbc_core::Error) -> Self {
        if let Some(details) = err.details() {
            log::error!("{details:?}");
        }
        match err {
            mongo_odbc_core::Error::ColIndexOutOfBounds(u) => ODBCError::InvalidDescriptorIndex(*u),
            e => ODBCError::Core(e.clone()),
        }
    }
}
