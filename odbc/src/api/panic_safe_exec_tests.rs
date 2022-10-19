use crate::{
    errors::ODBCError,
    handles::definitions::{MongoHandle, MongoHandleRef, Statement, StatementState},
    panic_safe_exec,
};
use odbc_sys::{HStmt, SqlReturn};
use std::{
    panic,
    sync::{mpsc, RwLock},
};

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
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));
        let sql_return = non_panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::SUCCESS, sql_return);
        unsafe {
            assert!((*stmt_handle)
                .as_statement()
                .unwrap()
                .read()
                .unwrap()
                .errors
                .is_empty());
        }
    }

    #[test]
    fn test_panic() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));
        let sql_return = panic_fn(stmt_handle as *mut _);
        assert_eq!(SqlReturn::ERROR, sql_return);
        unsafe {
            assert_eq!(
                format!("Panic(\"panic test\\nOk(\\\"in file 'odbc/src/api/panic_safe_exec_tests.rs' at line 19\\\")\")"),
                format!(
                    "{:?}",
                    (*stmt_handle)
                        .as_statement()
                        .unwrap()
                        .read()
                        .unwrap()
                        .errors[0]
                ),
            )
        }
    }
}
