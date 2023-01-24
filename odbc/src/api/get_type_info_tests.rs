use crate::{definitions::DiagType, handles::definitions::*, SQLGetDiagFieldW, SQLGetTypeInfo};
use bson::Bson;
use mongo_odbc_core::SqlDataType;
use odbc_sys::{HandleType::Stmt, SqlReturn};

const INVALID_SQL_TYPE: &str = "HY004\0";

mod unit {
    use super::*;
    use mongo_odbc_core::Error;
    use std::ffi::c_void;

    fn validate_result_set(data_type: SqlDataType, expectations: Vec<&str>) {
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let get_next = |stmt: &Statement| -> Result<bool, Error> {
                stmt.mongo_statement
                    .write()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .next(None)
            };
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(handle as *mut _, data_type as i16)
            );
            let stmt = (*handle).as_statement().unwrap();
            // for each expectation, check that calling next succeeds and we get the expected value
            expectations.iter().for_each(|name| {
                let result = get_next(stmt);
                assert!(result.unwrap());
                assert_eq!(
                    Bson::String((*name).to_string()),
                    stmt.mongo_statement
                        .write()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .get_value(1)
                        .unwrap()
                        .unwrap()
                )
            });

            // check there are no additional results in the set that were not expected
            let result = get_next(stmt);
            assert!(!result.unwrap());
        }
    }

    #[test]
    fn test_all_types() {
        let expectations = vec![
            "bool",
            "long",
            "binData",
            "array",
            "bson",
            "dbPointer",
            "decimal",
            "javascript",
            "javascriptWithScope",
            "maxKey",
            "minKey",
            "null",
            "object",
            "objectId",
            "symbol",
            "timestamp",
            "undefined",
            "int",
            "double",
            "string",
            "date",
        ];
        validate_result_set(SqlDataType::UNKNOWN_TYPE, expectations);
    }

    #[test]
    fn test_invalid_type_error() {
        // Test that a sql data type that is not defined in the enum yields the correct error
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            assert_eq!(SqlReturn::ERROR, SQLGetTypeInfo(handle as *mut _, 100));
            // use SQLGetDiagField to retreive and assert correct error message
            let message_text = &mut [0u16; 6] as *mut _ as *mut c_void;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    Stmt,
                    handle as *mut _,
                    1,
                    DiagType::SQL_DIAG_SQLSTATE as i16,
                    message_text,
                    6,
                    &mut 0
                )
            );
            assert_eq!(
                INVALID_SQL_TYPE,
                String::from_utf16(&*(message_text as *const [u16; 6])).unwrap()
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
                SQLGetTypeInfo(handle as *mut _, SqlDataType::INTEGER as i16)
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
