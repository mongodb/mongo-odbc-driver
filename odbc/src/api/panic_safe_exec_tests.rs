use crate::{
    errors::ODBCError,
    handles::definitions::{MongoHandle, MongoHandleRef, Statement, StatementState},
    panic_safe_exec,
};
use odbc_sys::{HStmt, SqlReturn};
use std::{panic, sync::mpsc};

mod unit {
    use super::*;
    fn non_panic_fn(stmt_handle: HStmt) -> SqlReturn {
        panic_safe_exec!(|| { SqlReturn::SUCCESS }, stmt_handle);
    }

    fn panic_fn(stmt_handle: HStmt) -> SqlReturn {
        panic_safe_exec!(|| { panic!("panic test") }, stmt_handle);
    }

    #[test]
    fn test_non_panic() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        let sql_return = non_panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::SUCCESS, sql_return);
        unsafe {
            assert!((*stmt_handle)
                .as_statement()
                .unwrap()
                .errors
                .read()
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn test_panic() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        let sql_return = panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::ERROR, sql_return);
        unsafe {
            let actual_error = format!(
                "{:?}",
                (*stmt_handle)
                    .as_statement()
                    .unwrap()
                    .errors
                    .read()
                    .unwrap()[0]
            );
            // Using a substring of the error because directory format differs for windows and linux
            // and to not depend on line number.
            assert_eq!(
                format!("Panic(\"panic test\\nOk(\\\"in file '"),
                &actual_error[0..33]
            )
        }
    }
}
