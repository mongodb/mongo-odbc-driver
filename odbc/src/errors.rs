const VENDOR_IDENTIFIER: &str = "MongoDB";

// SQL states
pub const HYC00: &str = "HYC00";
pub const HY024: &str = "HY024";

#[derive(Debug)]
pub enum ODBCError {
    Unimplemented(&'static str),
    InvalidAttrValue(&'static str)
}

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) => HYC00,
            ODBCError::InvalidAttrValue(_) => HY024,
        }
    }
    pub fn get_error_message(&self) -> String {
        match self {
            ODBCError::Unimplemented(fn_name) => format!(
                "[{}][API] The feature {} is not implemented",
                VENDOR_IDENTIFIER, fn_name
            ),
            ODBCError::InvalidAttrValue(attr) => format!(
                "[{}][API] Invalid value for attribute {}",
                VENDOR_IDENTIFIER, attr
            ),
        }
    }
    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Functions that return these errors don't interact with MongoDB,
            // and so the driver returns 0 since it doesn't have a native error
            // code to propagate.
            ODBCError::Unimplemented(_) | ODBCError::InvalidAttrValue(_) => 0,
        }
    }
}
