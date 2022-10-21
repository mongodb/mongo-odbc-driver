use constants::{
    GENERAL_ERROR, INVALID_ATTR_VALUE, INVALID_CURSOR_STATE, INVALID_DESCRIPTOR_INDEX,
    NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, OPTION_CHANGED, RIGHT_TRUNCATED, UNABLE_TO_CONNECT,
    UNSUPPORTED_FIELD_DESCRIPTOR, VENDOR_IDENTIFIER,
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
        "[{}][API] Buffer size '{0}' not large enough for string",
        VENDOR_IDENTIFIER
    )]
    OutStringTruncated(usize),
    #[error(
        "[{}][API] Invalid value for attribute {0}, changed to {1}",
        VENDOR_IDENTIFIER
    )]
    OptionValueChanged(&'static str, &'static str),
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
            | ODBCError::UnsupportedConnectionAttribute(_) => NOT_IMPLEMENTED,
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
            ODBCError::InvalidDescriptorIndex(_) => INVALID_DESCRIPTOR_INDEX,
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
            | ODBCError::UnsupportedFieldDescriptor(_) => 0,
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
