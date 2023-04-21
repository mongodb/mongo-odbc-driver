use crate::{definitions::DiagType, handles::definitions::*, SQLGetDiagFieldW, SQLGetTypeInfoW};
use mongo_odbc_core::SqlDataType;
use odbc_sys::{HandleType::Stmt, SqlReturn};

const INVALID_SQL_TYPE: &str = "HY004\0";

mod unit {
    use super::*;
    use cstr::WideChar;
    use std::{ffi::c_void, mem::size_of};

    #[test]
    fn test_invalid_type_error() {
        // Test that a sql data type that is not defined in the enum yields the correct error
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            assert_eq!(SqlReturn::ERROR, SQLGetTypeInfoW(handle as *mut _, 100));
            // use SQLGetDiagField to retreive and assert correct error message
            let message_text = &mut [0; 6] as *mut _ as *mut c_void;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    Stmt,
                    handle as *mut _,
                    1,
                    DiagType::SQL_DIAG_SQLSTATE as i16,
                    message_text,
                    6 * size_of::<WideChar>() as i16,
                    &mut 0
                )
            );
            assert_eq!(
                INVALID_SQL_TYPE,
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 6]))
            );
        }
    }

    #[test]
    fn test_invalid_cursor_state_error() {
        // checks for invalid cursor state when calling get_value before next
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfoW(handle as *mut _, SqlDataType::INTEGER as i16)
            );
            let value = stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_value(1);
            assert!(value.is_err());
        }
    }
}
