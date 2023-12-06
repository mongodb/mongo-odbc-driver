use constants::{
    OdbcState, CONNECTION_NOT_OPEN, FRACTIONAL_TRUNCATION, GENERAL_ERROR, GENERAL_WARNING,
    INDICATOR_VARIABLE_REQUIRED, INTEGRAL_TRUNCATION, INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER,
    INVALID_ATTR_VALUE, INVALID_CHARACTER_VALUE, INVALID_COLUMN_NUMBER, INVALID_CURSOR_STATE,
    INVALID_DATETIME_FORMAT, INVALID_DESCRIPTOR_INDEX, INVALID_INFO_TYPE_VALUE, INVALID_SQL_TYPE,
    NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, NO_RESULTSET, OPTION_CHANGED, PROGRAM_TYPE_OUT_OF_RANGE,
    RESTRICTED_DATATYPE, RIGHT_TRUNCATED, UNSUPPORTED_FIELD_DESCRIPTOR, VENDOR_IDENTIFIER,
};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ODBCError {
    #[error("[{}][API] {0}", VENDOR_IDENTIFIER)]
    General(&'static str),
    #[error("[{}][API] {0}", VENDOR_IDENTIFIER)]
    GeneralWarning(String),
    #[error("[{}][API] Caught panic: {0}", VENDOR_IDENTIFIER)]
    Panic(String),
    #[error("[{}][API] The feature {0} is not implemented", VENDOR_IDENTIFIER)]
    Unimplemented(&'static str),
    #[error("[{}][API] The data type {0} is not implemented", VENDOR_IDENTIFIER)]
    UnimplementedDataType(String),
    #[error(
        "[{}][API] The driver connect option {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedDriverConnectOption(String),
    #[error(
        "[{}][API] The connection attribute {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedConnectionAttribute(String),
    #[error(
        "[{}][API] The statement attribute {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedStatementAttribute(String),
    #[error(
        "[{}][API] A schema pattern was specified, and the driver does not support schemas",
        VENDOR_IDENTIFIER
    )]
    UnsupportedFieldSchema(),
    #[error(
        "[{}][API] The field descriptor value {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedFieldDescriptor(String),
    #[error(
        "[{}][API] Retrieving value for infoType {0} is not implemented yet",
        VENDOR_IDENTIFIER
    )]
    UnsupportedInfoTypeRetrieval(String),
    #[error("[{}][API] InfoType {0} out of range", VENDOR_IDENTIFIER)]
    UnknownInfoType(String),
    #[error(
        "[{}][API] Indicator variable was null when null data was accessed",
        VENDOR_IDENTIFIER
    )]
    IndicatorVariableRequiredButNotSupplied,
    #[error("[{}][API] The field index {0} is out of bounds", VENDOR_IDENTIFIER)]
    InvalidDescriptorIndex(u16),
    #[error("[{}][API] The column index {0} is out of bounds", VENDOR_IDENTIFIER)]
    InvalidColumnNumber(u16),
    #[error("[{}][API] No ResultSet", VENDOR_IDENTIFIER)]
    InvalidCursorState,
    #[error("[{}][API] Invalid SQL Type: {0}", VENDOR_IDENTIFIER)]
    InvalidSqlType(String),
    #[error("[{}][API] Invalid handle type, expected {0}", VENDOR_IDENTIFIER)]
    InvalidHandleType(&'static str),
    #[error("[{}][API] Invalid value for attribute {0}", VENDOR_IDENTIFIER)]
    InvalidAttrValue(&'static str),
    #[error("[{}][API] Invalid attribute identifier {0}", VENDOR_IDENTIFIER)]
    InvalidAttrIdentifier(i32),
    #[error("[{}][API] Invalid target type {0}", VENDOR_IDENTIFIER)]
    InvalidTargetType(i16),
    #[error(
        "[{}][API] Missing property \"Driver\" or \"DSN\" in connection string",
        VENDOR_IDENTIFIER
    )]
    MissingDriverOrDSNProperty,
    #[error(
        "[{}][API] Buffer size \"{0}\" not large enough for data",
        VENDOR_IDENTIFIER
    )]
    OutStringTruncated(usize),
    #[error(
        "[{}][API] floating point data \"{0}\" was truncated to fixed point",
        VENDOR_IDENTIFIER
    )]
    FractionalTruncation(String),
    #[error(
        "[{}][API] fractional seconds for data \"{0}\" was truncated to nanoseconds",
        VENDOR_IDENTIFIER
    )]
    FractionalSecondsTruncation(String),
    #[error(
        "[{}][API] fractional seconds data truncated from \"{0}\"",
        VENDOR_IDENTIFIER
    )]
    SecondsTruncation(String),
    #[error(
        "[{}][API] datetime data \"{0}\" was truncated to date",
        VENDOR_IDENTIFIER
    )]
    TimeTruncation(String),
    #[error(
        "[{}][API] integral data \"{0}\" was truncated due to overflow",
        VENDOR_IDENTIFIER
    )]
    IntegralTruncation(String),
    #[error("[{}][API] invalid datetime format", VENDOR_IDENTIFIER)]
    InvalidDatetimeFormat,
    #[error(
        "[{}][API] invalid character value for cast to type: {0}",
        VENDOR_IDENTIFIER
    )]
    InvalidCharacterValue(&'static str),
    #[error(
        "[{}][API] Invalid value for attribute {0}, changed to {1}",
        VENDOR_IDENTIFIER
    )]
    OptionValueChanged(&'static str, &'static str),
    #[error(
        "[{}][API] BSON type {0} cannot be converted to ODBC type {1}",
        VENDOR_IDENTIFIER
    )]
    RestrictedDataType(&'static str, &'static str),
    #[error("[{}][API] No resultset for statement", VENDOR_IDENTIFIER)]
    NoResultSet,
    #[error("Connection not open")]
    ConnectionNotOpen,
    #[error("[{}][Core] {0}", VENDOR_IDENTIFIER)]
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
            | ODBCError::UnsupportedInfoTypeRetrieval(_) => NOT_IMPLEMENTED,
            ODBCError::General(_) | ODBCError::Panic(_) => GENERAL_ERROR,
            ODBCError::GeneralWarning(_) => GENERAL_WARNING,
            ODBCError::Core(c) => c.get_sql_state(),
            ODBCError::InvalidAttrValue(_) => INVALID_ATTR_VALUE,
            ODBCError::InvalidAttrIdentifier(_) => INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER,
            ODBCError::InvalidCursorState => INVALID_CURSOR_STATE,
            ODBCError::InvalidHandleType(_) => NOT_IMPLEMENTED,
            ODBCError::InvalidTargetType(_) => PROGRAM_TYPE_OUT_OF_RANGE,
            ODBCError::OptionValueChanged(_, _) => OPTION_CHANGED,
            ODBCError::OutStringTruncated(_) => RIGHT_TRUNCATED,
            ODBCError::MissingDriverOrDSNProperty => NO_DSN_OR_DRIVER,
            ODBCError::UnsupportedFieldDescriptor(_) => UNSUPPORTED_FIELD_DESCRIPTOR,
            ODBCError::InvalidDescriptorIndex(_) => INVALID_DESCRIPTOR_INDEX,
            ODBCError::InvalidColumnNumber(_) => INVALID_COLUMN_NUMBER,
            ODBCError::InvalidSqlType(_) => INVALID_SQL_TYPE,
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
            | ODBCError::InvalidCursorState
            | ODBCError::InvalidHandleType(_)
            | ODBCError::InvalidTargetType(_)
            | ODBCError::MissingDriverOrDSNProperty
            | ODBCError::OutStringTruncated(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::UnsupportedConnectionAttribute(_)
            | ODBCError::UnsupportedStatementAttribute(_)
            | ODBCError::UnsupportedFieldSchema()
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
        match err {
            mongo_odbc_core::Error::ColIndexOutOfBounds(u) => ODBCError::InvalidDescriptorIndex(u),
            e => ODBCError::Core(e),
        }
    }
}

impl From<&mongo_odbc_core::Error> for ODBCError {
    fn from(err: &mongo_odbc_core::Error) -> Self {
        match err {
            mongo_odbc_core::Error::ColIndexOutOfBounds(u) => ODBCError::InvalidDescriptorIndex(*u),
            e => ODBCError::Core(e.clone()),
        }
    }
}
