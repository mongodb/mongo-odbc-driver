const VENDOR_IDENTIFIER: &str = "MongoDB";

// SQL states
pub const HYC00: &str = "HYC00";

#[derive(Debug)]
pub enum ODBCError {
    Unimplemented(&'static str),
}

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) => HYC00,
        }
    }
    pub fn get_error_message(&self) -> String {
        match self {
            ODBCError::Unimplemented(fn_name) => format!(
                "[{}][API] The feature {} is not implemented",
                VENDOR_IDENTIFIER, fn_name
            ),
        }
    }
    pub fn get_native_err_code(&self) -> i32 {
        match self {
            // Since unimplemented functions don't interact with MongoDB,
            // the driver doesn't have a native error code to propagate,
            // so we return 0.
            ODBCError::Unimplemented(_) => 0,
        }
    }
}
