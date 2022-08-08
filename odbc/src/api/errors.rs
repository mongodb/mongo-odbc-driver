const VENDOR_IDENTIFIER: &str = "MongoDB";

// SQL states
pub const HYC00: &str = "HYC00";
pub const HY024: &str = "HY024";
pub const _01S02: &str = "01S02";

#[derive(Debug)]
pub enum ODBCError {
    Unimplemented(&'static str),
    InvalidHandleType(&'static str),
    InvalidAttrValue(&'static str),
    CoreError(mongo_odbc_core::Error),
    OptionValueChanged(&'static str, &'static str),
}

impl From<mongo_odbc_core::Error> for ODBCError {
    fn from(me: mongo_odbc_core::Error) -> ODBCError {
        ODBCError::CoreError(me)
    }
}

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) => HYC00,
            ODBCError::CoreError(_) => HYC00,
            ODBCError::InvalidAttrValue(_) => HY024,
            ODBCError::InvalidHandleType(_) => HYC00,
            ODBCError::OptionValueChanged(_, _) => _01S02,
        }
    }
    pub fn get_error_message(&self) -> String {
        match self {
            ODBCError::Unimplemented(fn_name) => format!(
                "[{}][API] The feature {} is not implemented",
                VENDOR_IDENTIFIER, fn_name
            ),
            ODBCError::CoreError(error) => format!("[{}][Core] {:?}", VENDOR_IDENTIFIER, error),
            ODBCError::InvalidHandleType(ty) => format!(
                "[{}][API] Invalid handle type, expected {}",
                VENDOR_IDENTIFIER, ty
            ),
            ODBCError::InvalidAttrValue(attr) => format!(
                "[{}][API] Invalid value for attribute {}",
                VENDOR_IDENTIFIER, attr
            ),
            ODBCError::OptionValueChanged(attr, value) => format!(
                "[{}][API] Invalid value for attribute {}, changed to {}",
                VENDOR_IDENTIFIER, attr, value
            ),
        }
    }
    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Functions that return these errors don't interact with MongoDB,
            // and so the driver returns 0 since it doesn't have a native error
            // code to propagate.
            ODBCError::Unimplemented(_)
            | ODBCError::InvalidAttrValue(_)
            | ODBCError::InvalidHandleType(_)
            | ODBCError::OptionValueChanged(_, _) => 0,
            ODBCError::CoreError(me) => me.code(),
        }
    }
}
