extern crate odbc_sys;
use odbc_sys::*;
use std::ops::FnOnce;
use std::ptr::null_mut;

fn execute_odbc_call<F>(validate_success: bool, func: F)
where
    F: FnOnce() -> SqlReturn,
{
    let outcome: SqlReturn = func();
    if validate_success {
        assert_eq!(SqlReturn::SUCCESS, outcome)
    } else if SqlReturn::ERROR == outcome {
        panic!("ODBC call failed. Outcome is SQL_ERROR")
        // TODO : Get the error information for the diagnostic records
    }
}

/// Setup flow.
/// This will allocate a new environment handle and set ODBC_VERSION and CONNECTION_POOLING environment attributes.
fn setup(validate_success: bool) -> odbc_sys::HEnv {
    /*
        Setup flow :
            SQLAllocHandle(SQL_HANDLE_ENV)
            SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
            SQLSetEnvAttr(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
    */

    let mut env: Handle = null_mut();

    unsafe {
        execute_odbc_call(validate_success, || {
            SQLAllocHandle(HandleType::Env, null_mut(), &mut env as *mut Handle)
        });

        execute_odbc_call(validate_success, || {
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::OdbcVersion,
                AttrOdbcVersion::Odbc3.into(),
                0,
            )
        });

        execute_odbc_call(validate_success, || {
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                AttrConnectionPooling::OnePerHenv.into(),
                0,
            )
        });
    }

    env as HEnv
}

/// Test PowerBI Setup sequence
#[test]
fn test_setup() {
    setup(true);
}

#[test]
fn test_env_cleanup() {
    // We need a handle to be able to test that freeing the handle work
    let env_handle: HEnv = setup(false);

    unsafe {
        // Verify that freeing the handle is working as expected
        execute_odbc_call(true, || {
            SQLFreeHandle(HandleType::Env, env_handle as Handle)
        });
    }
}
