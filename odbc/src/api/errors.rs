use thiserror::Error;

const VENDOR_IDENTIFIER: &str = "MongoDB";

// SQL states
pub const HYC00: &str = "HYC00";
pub const HY024: &str = "HY024";
pub const _01S02: &str = "01S02";

#[derive(Debug, Error)]
pub enum ODBCError {
    #[error("[{}][API] The feature {0} is not implemented", VENDOR_IDENTIFIER)]
    Unimplemented(&'static str),
    #[error("[{}][API] Invalid Uri {0}", VENDOR_IDENTIFIER)]
    InvalidUriFormat(String),
    #[error("[{}][API] Invalid handle type, expected {0}", VENDOR_IDENTIFIER)]
    InvalidHandleType(&'static str),
    #[error("[{}][API] Invalid value for attribute {0}", VENDOR_IDENTIFIER)]
    InvalidAttrValue(&'static str),
    #[error(
        "[{}][API] Invalid value for attribute {0}, changed to {1}",
        VENDOR_IDENTIFIER
    )]
    OptionValueChanged(&'static str, &'static str),
    #[error("[{}][Core] {0}", VENDOR_IDENTIFIER)]
    CoreError(#[from] mongo_odbc_core::Error),
}

pub type Result<T> = std::result::Result<T, ODBCError>;

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) => HYC00,
            ODBCError::CoreError(_) => HYC00,
            ODBCError::InvalidUriFormat(_) | ODBCError::InvalidAttrValue(_) => HY024,
            ODBCError::InvalidHandleType(_) => HYC00,
            ODBCError::OptionValueChanged(_, _) => _01S02,
        }
    }
    pub fn get_error_message(&self) -> String {
        format!("{}", self)
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
            | ODBCError::OptionValueChanged(_, _) => 0,
            ODBCError::CoreError(me) => me.code(),
        }
    }
}
