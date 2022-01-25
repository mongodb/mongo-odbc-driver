const VENDOR_IDENTIFIER: &str = "MongoDB";

// SQL states
pub const HYC00: &str = "HYC00";
pub const HY024: &str = "HY024";
pub const _01S02: &str = "01S02";

#[derive(Debug)]
pub enum ODBCError {
    Unimplemented(&'static str),
    InvalidAttrValue(&'static str),
    OptionValueChanged(&'static str, &'static str),
}

impl ODBCError {
    pub fn get_sql_state(&self) -> &str {
        match self {
            ODBCError::Unimplemented(_) => HYC00,
            ODBCError::InvalidAttrValue(_) => HY024,
            ODBCError::OptionValueChanged(_, _) => _01S02,
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
            | ODBCError::OptionValueChanged(_, _) => 0,
        }
    }
}
