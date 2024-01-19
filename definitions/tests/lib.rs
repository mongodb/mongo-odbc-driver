//! Contains test for the ffi layer
extern crate definitions;
use definitions::*;
use std::ptr::null_mut;

#[test]
fn allocate_environment() {
    let mut env: Handle = null_mut();
    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV,
                null_mut(),
                &mut env as *mut Handle
            )
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_ENV, env)
        );
    }
}

#[test]
fn allocate_connection() {
    let mut env: Handle = null_mut();
    let mut conn: Handle = null_mut();

    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV,
                null_mut(),
                &mut env as *mut Handle
            )
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::SQL_ATTR_ODBC_VERSION,
                AttrOdbcVersion::SQL_OV_ODBC3.into(),
                0
            )
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(HandleType::SQL_HANDLE_DBC, env, &mut conn as *mut Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_DBC, conn)
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_ENV, env)
        );
    }
}

#[test]
fn allocate_connection_error() {
    let mut env: Handle = null_mut();
    let mut conn: Handle = null_mut();

    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV,
                null_mut(),
                &mut env as *mut Handle
            )
        );

        // Allocating connection without setting ODBC Version first should result in an error
        assert_eq!(
            SqlReturn::ERROR,
            SQLAllocHandle(HandleType::SQL_HANDLE_DBC, env, &mut conn as *mut Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_ENV, env)
        );
    }
}
