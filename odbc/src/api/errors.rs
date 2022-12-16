use constants::{
    FRACTIONAL_TRUNCATION, GENERAL_ERROR, INDICATOR_VARIABLE_REQUIRED, INTEGRAL_TRUNCATION,
    INVALID_ATTR_VALUE, INVALID_CHARACTER_VALUE, INVALID_CURSOR_STATE, INVALID_DATETIME_FORMAT,
    INVALID_DESCRIPTOR_INDEX, INVALID_INFO_TYPE_VALUE, NOT_IMPLEMENTED, NO_DSN_OR_DRIVER,
    NO_RESULTSET, OPTION_CHANGED, RESTRICTED_DATATYPE, RIGHT_TRUNCATED, UNABLE_TO_CONNECT,
    UNSUPPORTED_DIAG_IDENTIFIER, UNSUPPORTED_FIELD_DESCRIPTOR, VENDOR_IDENTIFIER,
};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ODBCError {
    #[error("[{}][API] {0}", VENDOR_IDENTIFIER)]
    General(&'static str),
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
        "[{}][API] The field descriptor value {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedFieldDescriptor(String),
    #[error(
        "[{}][API] The diag identifier value {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedDiagIdentifier(String),
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
    #[error("[{}][API] No ResultSet", VENDOR_IDENTIFIER)]
    InvalidCursorState,
    #[error("[{}][API] Invalid Uri: {0}", VENDOR_IDENTIFIER)]
    InvalidUriFormat(String),
    #[error("[{}][API] Invalid handle type, expected {0}", VENDOR_IDENTIFIER)]
    InvalidHandleType(&'static str),
    #[error("[{}][API] Invalid value for attribute {0}", VENDOR_IDENTIFIER)]
    InvalidAttrValue(&'static str),
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
    #[error("[{}][API] invalid datetime format: \"{0}\"", VENDOR_IDENTIFIER)]
    InvalidDatetimeFormat(String),
    #[error(
        "[{}][API] invalid character value: \"{0}\" for cast to type: {1}",
        VENDOR_IDENTIFIER
    )]
    InvalidCharacterValue(String, &'static str),
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
    #[error("[{}][Core] {0}", VENDOR_IDENTIFIER)]
    Core(mongo_odbc_core::Error),
}

pub type Result<T> = std::result::Result<T, ODBCError>;

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_)
            | ODBCError::UnimplementedDataType(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::UnsupportedConnectionAttribute(_)
            | ODBCError::UnsupportedInfoTypeRetrieval(_) => NOT_IMPLEMENTED,
            ODBCError::General(_) | ODBCError::Panic(_) => GENERAL_ERROR,
            ODBCError::Core(c) => c.get_sql_state(),
            ODBCError::InvalidUriFormat(_) => UNABLE_TO_CONNECT,
            ODBCError::InvalidAttrValue(_) => INVALID_ATTR_VALUE,
            ODBCError::InvalidCursorState => INVALID_CURSOR_STATE,
            ODBCError::InvalidHandleType(_) => NOT_IMPLEMENTED,
            ODBCError::OptionValueChanged(_, _) => OPTION_CHANGED,
            ODBCError::OutStringTruncated(_) => RIGHT_TRUNCATED,
            ODBCError::MissingDriverOrDSNProperty => NO_DSN_OR_DRIVER,
            ODBCError::UnsupportedFieldDescriptor(_) => UNSUPPORTED_FIELD_DESCRIPTOR,
            ODBCError::UnsupportedDiagIdentifier(_) => UNSUPPORTED_DIAG_IDENTIFIER,
            ODBCError::InvalidDescriptorIndex(_) => INVALID_DESCRIPTOR_INDEX,
            ODBCError::RestrictedDataType(_, _) => RESTRICTED_DATATYPE,
            ODBCError::FractionalTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::FractionalSecondsTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::SecondsTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::TimeTruncation(_) => FRACTIONAL_TRUNCATION,
            ODBCError::IntegralTruncation(_) => INTEGRAL_TRUNCATION,
            ODBCError::InvalidDatetimeFormat(_) => INVALID_DATETIME_FORMAT,
            ODBCError::InvalidCharacterValue(_, _) => INVALID_CHARACTER_VALUE,
            ODBCError::IndicatorVariableRequiredButNotSupplied => INDICATOR_VARIABLE_REQUIRED,
            ODBCError::NoResultSet => NO_RESULTSET,
            ODBCError::UnknownInfoType(_) => INVALID_INFO_TYPE_VALUE,
        }
    }

    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Functions that return these errors don't interact with MongoDB,
            // and so the driver returns 0 since it doesn't have a native error
            // code to propagate.
            ODBCError::Unimplemented(_)
            | ODBCError::General(_)
            | ODBCError::Panic(_)
            | ODBCError::UnimplementedDataType(_)
            | ODBCError::InvalidUriFormat(_)
            | ODBCError::InvalidAttrValue(_)
            | ODBCError::InvalidCursorState
            | ODBCError::InvalidHandleType(_)
            | ODBCError::MissingDriverOrDSNProperty
            | ODBCError::OutStringTruncated(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::UnsupportedConnectionAttribute(_)
            | ODBCError::OptionValueChanged(_, _)
            | ODBCError::InvalidDescriptorIndex(_)
            | ODBCError::RestrictedDataType(_, _)
            | ODBCError::IndicatorVariableRequiredButNotSupplied
            | ODBCError::FractionalTruncation(_)
            | ODBCError::FractionalSecondsTruncation(_)
            | ODBCError::SecondsTruncation(_)
            | ODBCError::TimeTruncation(_)
            | ODBCError::IntegralTruncation(_)
            | ODBCError::InvalidDatetimeFormat(_)
            | ODBCError::UnsupportedFieldDescriptor(_)
            | ODBCError::UnsupportedDiagIdentifier(_)
            | ODBCError::InvalidCharacterValue(_, _)
            | ODBCError::NoResultSet
            | ODBCError::UnsupportedInfoTypeRetrieval(_)
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
