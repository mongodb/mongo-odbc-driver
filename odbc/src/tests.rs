use crate::{
    handles::{Connection, ConnectionState, Env, EnvState, MongoHandle, Statement, StatementState},
    SQLAllocHandle, SQLFreeHandle,
};
use odbc_sys::{Handle, HandleType, SqlReturn};
use std::sync::RwLock;

const UNIMPLEMENTED_FUNC: &str = "HYC00\0";

#[test]
fn env_alloc_free() {
    unsafe {
        let mut handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Env,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            EnvState::Allocated,
            (*handle).as_env().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Env,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn connection_alloc_free() {
    unsafe {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        let mut handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(Connection::with_state(
            std::ptr::null_mut(),
            ConnectionState::Allocated,
        )));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::Allocated,
            (*handle).as_connection().unwrap().read().unwrap().state
        );
        assert_eq!(
            1,
            (*env_handle)
                .as_env()
                .unwrap()
                .read()
                .unwrap()
                .connections
                .len()
        );
        assert_eq!(
            EnvState::ConnectionAllocated,
            (*env_handle).as_env().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            0,
            (*env_handle)
                .as_env()
                .unwrap()
                .read()
                .unwrap()
                .connections
                .len()
        );
        assert_eq!(
            EnvState::Allocated,
            (*env_handle).as_env().unwrap().read().unwrap().state
        );
    }
}

#[test]
fn statement_alloc_free() {
    unsafe {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        let conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(
            Connection::with_state(env_handle, ConnectionState::Allocated),
        ));

        let mut handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Stmt,
                conn_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            StatementState::Allocated,
            (*handle).as_statement().unwrap().write().unwrap().state
        );
        assert_eq!(
            1,
            (*conn_handle)
                .as_connection()
                .unwrap()
                .read()
                .unwrap()
                .statements
                .len()
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            0,
            (*conn_handle)
                .as_connection()
                .unwrap()
                .read()
                .unwrap()
                .statements
                .len()
        );
    }
}

#[test]
fn invalid_free() {
    unsafe {
        let mut env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        let env_handle_ptr: *mut _ = &mut env_handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Env,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(env_handle_ptr),
            )
        );
        assert_eq!(
            EnvState::Allocated,
            (*env_handle).as_env().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );

        let mut conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(
            Connection::with_state(env_handle, ConnectionState::Allocated),
        ));
        let conn_handle_ptr: *mut _ = &mut conn_handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(conn_handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::Allocated,
            (*conn_handle)
                .as_connection()
                .unwrap()
                .read()
                .unwrap()
                .state
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Env,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );

        // Free for real so we don't leak. Note we must free the Connection before the Env or we
        // will violate ASAN!
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Env,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );
    }
}

#[test]
fn invalid_alloc() {
    unsafe {
        let mut handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        let handle_ptr: *mut _ = &mut handle;
        // first check null ptrs for the two handles that require parent handles
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Dbc,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Stmt,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );

        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

        // now test wrong parent handle type (Dbc needs Env, and Stmt needs Connection).
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Dbc,
                stmt_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Stmt,
                stmt_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
    }
}

mod get_diag_rec {
    use crate::{
        errors::ODBCError,
        handles::{
            Connection, ConnectionState, Env, EnvState, MongoHandle, Statement, StatementState,
        },
        tests::UNIMPLEMENTED_FUNC,
        SQLGetDiagRecW,
    };
    use odbc_sys::{HandleType, SqlReturn};
    use std::sync::RwLock;

    #[test]
    fn simple() {
        fn validate_diag_rec(handle_type: HandleType, handle: *mut MongoHandle) {
            const ERROR_MESSAGE: &str =
                "[MongoDB][API] The feature SQLDrivers is not implemented\0";

            // Initialize buffers
            let sql_state = &mut [0u16; 6] as *mut _;
            // Note: len(ERROR_MESSAGE) = 57
            let message_text = &mut [0u16; 57] as *mut _;
            let text_length_ptr = &mut 0;
            let native_err_ptr = &mut 0;

            unsafe { (*handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers")) }
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagRecW(
                    handle_type,
                    handle as *mut _,
                    1,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    60, // Some number >= 57
                    text_length_ptr,
                )
            );
            assert_eq!(UNIMPLEMENTED_FUNC, unsafe {
                String::from_utf16(&*(sql_state as *const [u16; 6])).unwrap()
            });
            assert_eq!(ERROR_MESSAGE, unsafe {
                String::from_utf16(&*(message_text as *const [u16; 57])).unwrap()
            });
            // Exclude the number of characters required for the null terminator
            assert_eq!(56, *text_length_ptr);
            assert_eq!(0, *native_err_ptr);
        }

        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        validate_diag_rec(HandleType::Env, env_handle);

        let conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(
            Connection::with_state(env_handle, ConnectionState::Allocated),
        ));
        validate_diag_rec(HandleType::Dbc, conn_handle);

        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));
        validate_diag_rec(HandleType::Stmt, stmt_handle);
    }

    #[test]
    fn error_message() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        // Initialize buffers
        let sql_state = &mut [0u16; 6] as *mut _;
        let message_text = &mut [0u16; 57] as *mut _;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;

        unsafe { (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers")) }
        // Buffer is too small to hold the entire error message and the null terminator
        // (0 < length < 57)
        assert_eq!(
            SqlReturn::SUCCESS_WITH_INFO,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                15,
                text_length_ptr
            )
        );
        assert_eq!(
            "[MongoDB][API]\0",
            String::from_utf16(unsafe { &*(message_text as *const [u16; 15]) }).unwrap()
        );
        // Error message string where some characters are composed of more than one byte.
        // 1 < RecNumber =< number of diagnostic records.
        unsafe { (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDriv✐𑜲")) }
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                2,
                sql_state,
                native_err_ptr,
                message_text,
                57,
                text_length_ptr
            )
        );
        assert_eq!(
            "[MongoDB][API] The feature SQLDriv✐𑜲 is not implemented\0",
            String::from_utf16(unsafe { &*(message_text as *const [u16; 57]) }).unwrap()
        );
    }

    #[test]
    fn invalid_ops() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        // Initialize buffers
        let sql_state = &mut [0u16; 6] as *mut _;
        let message_text = &mut [0u16; 57] as *mut _;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;

        unsafe { (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers")) }
        // Buffer length < 0
        assert_eq!(
            SqlReturn::ERROR,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                -1,
                text_length_ptr
            )
        );
        // Record number <= 0
        assert_eq!(
            SqlReturn::ERROR,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                0,
                sql_state,
                native_err_ptr,
                message_text,
                57,
                text_length_ptr
            )
        );
        // Record number > number of diagnostic records
        assert_eq!(
            SqlReturn::NO_DATA,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                3,
                sql_state,
                native_err_ptr,
                message_text,
                5,
                text_length_ptr
            )
        );
    }
}
mod env_attributes {
    use crate::{
        handles::{Env, EnvState, MongoHandle},
        tests::UNIMPLEMENTED_FUNC,
        SQLGetDiagRecW, SQLGetEnvAttrW, SQLSetEnvAttrW,
    };
    use odbc_sys::{
        AttrConnectionPooling, AttrCpMatch, AttrOdbcVersion, EnvironmentAttribute, HEnv,
        HandleType, Integer, Pointer, SqlReturn,
    };
    use std::{ffi::c_void, sync::RwLock};

    const INVALID_ATTR_VALUE: &str = "HY024\0";

    #[test]
    fn get_set_odbc_version() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        // SQLSetEnvAttrW(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3_80)
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::OdbcVersion,
                Pointer::from(AttrOdbcVersion::Odbc3_80),
                0
            )
        );

        // SQLGetEnvAttrW(SQL_ATTR_ODBC_VERSION)
        let attr_buffer = Box::into_raw(Box::new(0));
        let string_length_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                env_handle as *mut _,
                EnvironmentAttribute::OdbcVersion,
                attr_buffer as Pointer,
                0,
                string_length_ptr
            )
        );
        assert_eq!(AttrOdbcVersion::Odbc3_80 as Integer, unsafe {
            attr_buffer.read()
        });
        assert_eq!(4, *string_length_ptr);

        // SQLSetEnvAttrW(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::OdbcVersion,
                Pointer::from(AttrOdbcVersion::Odbc3),
                0
            )
        );

        unsafe { Box::from_raw(attr_buffer) };
    }

    #[test]
    fn get_set_output_nts() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        // SQLSetEnvAttrW(SQL_ATTR_OUTPUT_NTS, SQL_TRUE)
        let should_output_nts = Box::into_raw(Box::new(1));
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::OutputNts,
                should_output_nts as Pointer,
                0
            )
        );

        // SQLGetEnvAttrW(SQL_ATTR_OUTPUT_NTS)
        let attr_buffer = Box::into_raw(Box::new(0));
        let string_length_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                env_handle as *mut _,
                EnvironmentAttribute::OutputNts,
                attr_buffer as Pointer,
                5,
                string_length_ptr
            )
        );
        assert_eq!(1, unsafe { attr_buffer.read() });
        assert_eq!(4, *string_length_ptr);

        // SQLSetEnvAttrW(SQL_ATTR_OUTPUT_NTS, SQL_FALSE)
        unsafe { *should_output_nts = 0 };
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::OutputNts,
                should_output_nts as Pointer,
                0
            )
        );

        // Verify that the attribute's value didn't change in the previous
        // failed call to SQLSetEnvAttrW by calling SQLGetEnvAttrW(SQL_ATTR_OUTPUT_NTS)
        let string_length_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                env_handle as *mut _,
                EnvironmentAttribute::OutputNts,
                attr_buffer as *mut c_void,
                5,
                string_length_ptr
            )
        );
        assert_eq!(1, unsafe { attr_buffer.read() });
        assert_eq!(4, *string_length_ptr);

        // SQLSetEnvAttrW(SQL_ATTR_OUTPUT_NTS, <invalid number>)
        unsafe { *should_output_nts = 2 };
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::OutputNts,
                should_output_nts as Pointer,
                0
            )
        );
        // Initialize buffers
        let sql_state = &mut [0u16; 6] as *mut _;
        let message_text = &mut [0u16; 64] as *mut _;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                100,
                text_length_ptr
            )
        );
        assert_eq!(INVALID_ATTR_VALUE, unsafe {
            String::from_utf16(&*(sql_state as *const [u16; 6])).unwrap()
        });
        assert_eq!(
            "[MongoDB][API] Invalid value for attribute OUTPUT_NTS=SQL_FALSE\0",
            unsafe { String::from_utf16(&*(message_text as *const [u16; 64])).unwrap() }
        );
        unsafe { Box::from_raw(should_output_nts) };
        unsafe { Box::from_raw(attr_buffer) };
    }

    #[test]
    fn get_set_connection_pool() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        // SQLGetEnvAttrW(SQL_ATTR_CONNECTION_POOLING)
        let attr_buffer = Box::into_raw(Box::new(0));
        let string_length_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                env_handle as *mut _,
                EnvironmentAttribute::ConnectionPooling,
                attr_buffer as Pointer,
                5,
                string_length_ptr
            )
        );
        assert_eq!(AttrConnectionPooling::Off as Integer, unsafe {
            attr_buffer.read()
        });
        assert_eq!(4, *string_length_ptr);
        unsafe { Box::from_raw(attr_buffer) };

        // SQLSetEnvAttrW(SQL_ATTR_CONNECTION_POOLING, SQL_CP_DRIVER_AWARE)
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                Pointer::from(AttrConnectionPooling::DriverAware),
                0
            )
        );
        // Initialize buffers
        let sql_state = &mut [0u16; 6] as *mut _;
        let message_text = &mut [0u16; 74] as *mut _;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                74,
                text_length_ptr
            )
        );
        assert_eq!(UNIMPLEMENTED_FUNC, unsafe {
            String::from_utf16(&*(sql_state as *const [u16; 6])).unwrap()
        });
        assert_eq!(
            "[MongoDB][API] The feature SQL_ATTR_CONNECTION_POOLING is not implemented\0",
            unsafe { String::from_utf16(&*(message_text as *const [u16; 74])).unwrap() }
        );
        // Exclude the number of characters required for the null terminator
        assert_eq!(73, *text_length_ptr);
        assert_eq!(0, *native_err_ptr);

        // SQLSetEnvAttrW(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                Pointer::from(AttrConnectionPooling::OnePerHenv),
                0
            )
        );

        // SQLSetEnvAttrW(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_DRIVER)
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                Pointer::from(AttrConnectionPooling::OnePerDriver),
                0
            )
        );
    }

    #[test]
    fn get_set_cp_match() {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));

        // SQLGetEnvAttrW(SQL_ATTR_CP_MATCH)
        let attr_buffer = Box::into_raw(Box::new(0));
        let string_length_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                env_handle as *mut _,
                EnvironmentAttribute::CpMatch,
                attr_buffer as Pointer,
                5,
                string_length_ptr
            )
        );
        assert_eq!(AttrCpMatch::Strict as Integer, unsafe {
            attr_buffer.read()
        });
        assert_eq!(4, *string_length_ptr);
        unsafe { Box::from_raw(attr_buffer) };

        // SQLSetEnvAttrW(SQL_ATTR_CP_MATCH, SQL_CP_STRICT_MATCH)
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::CpMatch,
                Pointer::from(AttrCpMatch::Strict),
                0
            )
        );

        // SQLSetEnvAttrW(SQL_ATTR_CP_MATCH, SQL_CP_RELAXED_MATCH)
        assert_eq!(
            SqlReturn::ERROR,
            SQLSetEnvAttrW(
                env_handle as HEnv,
                EnvironmentAttribute::CpMatch,
                Pointer::from(AttrCpMatch::Relaxed),
                0
            )
        );
        // Initialize buffers
        let sql_state = &mut [0u16; 6] as *mut _;
        let message_text = &mut [0u16; 67] as *mut _;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRecW(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                100,
                text_length_ptr
            )
        );
        assert_eq!(UNIMPLEMENTED_FUNC, unsafe {
            String::from_utf16(&*(sql_state as *const [u16; 6])).unwrap()
        });
        assert_eq!(
            "[MongoDB][API] The feature SQL_CP_RELAXED_MATCH is not implemented\0",
            unsafe { String::from_utf16(&*(message_text as *const [u16; 67])).unwrap() }
        );
        // Exclude the number of characters required for the null terminator
        assert_eq!(66, *text_length_ptr);
        assert_eq!(0, *native_err_ptr);
    }
}
