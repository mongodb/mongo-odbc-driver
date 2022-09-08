use constants::{
    INVALID_ATTR_VALUE, NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, OPTION_CHANGED, RIGHT_TRUNCATED,
    UNABLE_TO_CONNECT, VENDOR_IDENTIFIER,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ODBCError {
    #[error("[{}][API] The feature {0} is not implemented", VENDOR_IDENTIFIER)]
    Unimplemented(&'static str),
    #[error(
        "[{}][API] The driver connect option {0} is not supported",
        VENDOR_IDENTIFIER
    )]
    UnsupportedDriverConnectOption(String),
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
    Core(#[from] mongo_odbc_core::Error),
}

pub type Result<T> = std::result::Result<T, ODBCError>;

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) | ODBCError::UnsupportedDriverConnectOption(_) => {
                NOT_IMPLEMENTED
            }
            ODBCError::Core(c) => c.get_sql_state(),
            ODBCError::InvalidUriFormat(_) => UNABLE_TO_CONNECT,
            ODBCError::InvalidAttrValue(_) => INVALID_ATTR_VALUE,
            ODBCError::InvalidHandleType(_) => NOT_IMPLEMENTED,
            ODBCError::OptionValueChanged(_, _) => OPTION_CHANGED,
            ODBCError::OutStringTruncated(_) => RIGHT_TRUNCATED,
            ODBCError::MissingDriverOrDSNProperty => NO_DSN_OR_DRIVER,
        }
    }

    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Functions that return these errors don't interact with MongoDB,
            // and so the driver returns 0 since it doesn't have a native error
            // code to propagate.
            ODBCError::Unimplemented(_)
            | ODBCError::InvalidUriFormat(_)
            | ODBCError::InvalidAttrValue(_)
            | ODBCError::InvalidHandleType(_)
            | ODBCError::MissingDriverOrDSNProperty
            | ODBCError::OutStringTruncated(_)
            | ODBCError::UnsupportedDriverConnectOption(_)
            | ODBCError::OptionValueChanged(_, _) => 0,
            ODBCError::Core(me) => me.code(),
        }
    }
}
