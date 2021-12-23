use odbc_sys::*;
use std::ptr::null_mut;

/// Setup flow.
/// This will allocate a new environment handle and set ODBC_VERSION and CONNECTION_POOLING environment attributes.
fn setup() -> odbc_sys::HEnv {
    /*
        Setup flow :
            SQLAllocHandle(SQL_HANDLE_ENV)
            SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
            SQLSetEnvAttr(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
    */

    let mut env: Handle = null_mut();

    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(HandleType::Env, null_mut(), &mut env as *mut Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::OdbcVersion,
                AttrOdbcVersion::Odbc3.into(),
                0,
            )
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                AttrConnectionPooling::OnePerHenv.into(),
                0,
            )
        );
    }

    env as HEnv
}

/// Test PowerBI Setup flow
#[test]
fn test_setup() {
    setup();
}

/// Test PowerBi environment clean-up
#[test]
fn test_env_cleanup() {
    // We need a handle to be able to test that freeing the handle work
    let env_handle: HEnv = setup();

    unsafe {
        // Verify that freeing the handle is working as expected
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::Env, env_handle as Handle)
        );
    }
}

// TODO : Add the other flows [SQL-639], [SQL-640], [SQL-641] [SQL-642]
