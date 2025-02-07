#![allow(
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

use crate::api::errors::ODBCError;
use crate::{handles::definitions::*, has_odbc_3_behavior, SQLAllocHandle, SQLFreeHandle};
use definitions::{AttrOdbcVersion, HDbc, HDesc, HEnv, HStmt, Handle, HandleType, SqlReturn};

#[test]
fn test_env_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            EnvState::Allocated,
            *(*handle).as_env().unwrap().state.read().unwrap(),
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_ENV,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn test_connection_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        let mut handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            std::ptr::null_mut(),
            ConnectionState::Allocated,
        ));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DBC,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::Allocated,
            *(*handle).as_connection().unwrap().state.read().unwrap()
        );
        assert_eq!(
            1,
            (*env_handle)
                .as_env()
                .unwrap()
                .connections
                .read()
                .unwrap()
                .len()
        );
        assert_eq!(
            EnvState::ConnectionAllocated,
            *(*env_handle).as_env().unwrap().state.read().unwrap(),
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DBC,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            0,
            (*env_handle)
                .as_env()
                .unwrap()
                .connections
                .read()
                .unwrap()
                .len()
        );
        assert_eq!(
            EnvState::Allocated,
            *(*env_handle).as_env().unwrap().state.read().unwrap(),
        );
    }
}

#[test]
fn test_statement_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        let conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
        ));

        let mut handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_STMT,
                conn_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            StatementState::Allocated,
            *(*handle).as_statement().unwrap().state.read().unwrap()
        );
        assert_eq!(
            1,
            (*conn_handle)
                .as_connection()
                .unwrap()
                .statements
                .read()
                .unwrap()
                .len()
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_STMT,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            0,
            (*conn_handle)
                .as_connection()
                .unwrap()
                .statements
                .read()
                .unwrap()
                .len()
        );
    }
}

#[test]
fn test_try_from_null_handles() {
    let null_handle: Handle = std::ptr::null_mut();
    let result = MongoHandleRef::try_from(null_handle);
    assert!(
        result.is_err(),
        "Expected conversion of a null Handle to fail"
    );
    assert!(
        matches!(result, Err(ODBCError::InvalidHandleType(msg)) if msg == "handle cannot be null"),
        "Unexpected error variant or message: got {result:?}",
    );

    let null_henv: HEnv = std::ptr::null_mut();
    let result = MongoHandleRef::try_from(null_henv);
    assert!(
        result.is_err(),
        "Expected conversion of a null HEnv to fail"
    );
    assert!(
        matches!(result, Err(ODBCError::InvalidHandleType(msg)) if msg == "handle cannot be null"),
        "Unexpected error variant or message: got {result:?}",
    );

    let null_hstmt: HStmt = std::ptr::null_mut();
    let result = MongoHandleRef::try_from(null_hstmt);
    assert!(
        result.is_err(),
        "Expected conversion of a null HStmt to fail"
    );
    assert!(
        matches!(result, Err(ODBCError::InvalidHandleType(msg)) if msg == "handle cannot be null"),
        "Unexpected error variant or message: got {result:?}",
    );

    let null_hdbc: HDbc = std::ptr::null_mut();
    let result = MongoHandleRef::try_from(null_hdbc);
    assert!(
        result.is_err(),
        "Expected conversion of a null HDbc to fail"
    );
    assert!(
        matches!(result, Err(ODBCError::InvalidHandleType(msg)) if msg == "handle cannot be null"),
        "Unexpected error variant or message: got {result:?}",
    );

    let null_hdesc: HDesc = std::ptr::null_mut();
    let result = MongoHandleRef::try_from(null_hdesc);
    assert!(
        result.is_err(),
        "Expected conversion of a null HDesc to fail"
    );
    assert!(
        matches!(result, Err(ODBCError::InvalidHandleType(msg)) if msg == "handle cannot be null"),
        "Unexpected error variant or message: got {result:?}",
    );
}

#[test]
fn test_descriptor_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        let conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
        ));

        let mut handle: *mut _ = &mut MongoHandle::Descriptor(Descriptor::with_state(
            std::ptr::null_mut(),
            DescriptorState::ExplicitlyAllocated,
        ));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DESC,
                conn_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            DescriptorState::ExplicitlyAllocated,
            *(*handle).as_descriptor().unwrap().state.read().unwrap()
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DESC,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn test_invalid_free() {
    unsafe {
        let mut env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let env_handle_ptr: *mut _ = &mut env_handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(env_handle_ptr),
            )
        );
        assert_eq!(
            EnvState::Allocated,
            *(*env_handle).as_env().unwrap().state.read().unwrap(),
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DBC,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_STMT,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DESC,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );

        let mut conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
        ));
        let conn_handle_ptr: *mut _ = &mut conn_handle;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DBC,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(conn_handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::Allocated,
            *(*conn_handle)
                .as_connection()
                .unwrap()
                .state
                .read()
                .unwrap()
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_ENV,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_STMT,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DESC,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );

        // Free for real so we don't leak. Note we must free the Connection before the Env or we
        // will violate ASAN!
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_DBC,
                std::mem::transmute::<*mut MongoHandle, Handle>(conn_handle),
            )
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::SQL_HANDLE_ENV,
                std::mem::transmute::<*mut MongoHandle, Handle>(env_handle),
            )
        );
    }
}

#[test]
fn test_invalid_alloc() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let handle_ptr: *mut _ = &mut handle;
        // first check null ptrs for the three handles that require parent handles
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DBC,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_STMT,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DESC,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );

        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        // now test wrong parent handle type (Dbc needs Env, and Stmt and Desc need Connection).
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DBC,
                stmt_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_STMT,
                stmt_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_DESC,
                stmt_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
    }
}

#[test]
fn test_odbc_ver() {
    // set up handles of each type with the underlying env handle being odbc 2
    let odbc_2_env_handle: &mut MongoHandle =
        &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    odbc_2_env_handle
        .as_env()
        .unwrap()
        .attributes
        .write()
        .unwrap()
        .odbc_ver = AttrOdbcVersion::SQL_OV_ODBC2;
    let odbc_2_conn_handle: &mut MongoHandle = &mut MongoHandle::Connection(
        Connection::with_state(odbc_2_env_handle, ConnectionState::Allocated),
    );
    let odbc_2_desc_handle: &mut _ = &mut MongoHandle::Descriptor(Descriptor::with_state(
        odbc_2_conn_handle,
        DescriptorState::ExplicitlyAllocated,
    ));
    let odbc_2_stmt_handle: &mut _ = &mut MongoHandle::Statement(Statement::with_state(
        odbc_2_conn_handle,
        StatementState::Allocated,
    ));

    // set up handles of each type with the underling env handle being the default odbc 3_80
    let odbc_3_env_handle: &mut MongoHandle =
        &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    let odbc_3_conn_handle: &mut MongoHandle = &mut MongoHandle::Connection(
        Connection::with_state(odbc_3_env_handle, ConnectionState::Allocated),
    );
    let odbc_3_desc_handle: &mut _ = &mut MongoHandle::Descriptor(Descriptor::with_state(
        odbc_3_conn_handle,
        DescriptorState::ExplicitlyAllocated,
    ));
    let odbc_3_stmt_handle: &mut _ = &mut MongoHandle::Statement(Statement::with_state(
        odbc_3_conn_handle,
        StatementState::Allocated,
    ));

    // assert correct types for all handles
    assert_eq!(
        odbc_2_env_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC2
    );
    assert_eq!(
        odbc_2_conn_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC2
    );
    assert_eq!(
        odbc_2_desc_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC2
    );
    assert_eq!(
        odbc_2_stmt_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC2
    );
    assert_eq!(
        odbc_3_env_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC3_80
    );
    assert_eq!(
        odbc_3_conn_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC3_80
    );
    assert_eq!(
        odbc_3_desc_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC3_80
    );
    assert_eq!(
        odbc_3_stmt_handle.get_odbc_version(),
        AttrOdbcVersion::SQL_OV_ODBC3_80
    );
}

#[test]
fn test_odbc_2_behavior() {
    // set up handles of each type with the underlying env handle being odbc 2
    let odbc_2_env_handle: &mut MongoHandle =
        &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    odbc_2_env_handle
        .as_env()
        .unwrap()
        .attributes
        .write()
        .unwrap()
        .odbc_ver = AttrOdbcVersion::SQL_OV_ODBC2;
    let odbc_2_conn_handle: &mut MongoHandle = &mut MongoHandle::Connection(
        Connection::with_state(odbc_2_env_handle, ConnectionState::Allocated),
    );
    let odbc_2_desc_handle: &mut _ = &mut MongoHandle::Descriptor(Descriptor::with_state(
        odbc_2_conn_handle,
        DescriptorState::ExplicitlyAllocated,
    ));
    let odbc_2_stmt_handle: &mut _ = &mut MongoHandle::Statement(Statement::with_state(
        odbc_2_conn_handle,
        StatementState::Allocated,
    ));

    // set up handles of each type with the underling env handle being the default odbc 3_80
    let odbc_3_env_handle: &mut MongoHandle =
        &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    let odbc_3_conn_handle: &mut MongoHandle = &mut MongoHandle::Connection(
        Connection::with_state(odbc_3_env_handle, ConnectionState::Allocated),
    );
    let odbc_3_desc_handle: &mut _ = &mut MongoHandle::Descriptor(Descriptor::with_state(
        odbc_3_conn_handle,
        DescriptorState::ExplicitlyAllocated,
    ));
    let odbc_3_stmt_handle: &mut _ = &mut MongoHandle::Statement(Statement::with_state(
        odbc_3_conn_handle,
        StatementState::Allocated,
    ));

    assert!(!has_odbc_3_behavior!(odbc_2_env_handle));
    assert!(!has_odbc_3_behavior!(odbc_2_conn_handle));
    assert!(!has_odbc_3_behavior!(odbc_2_desc_handle));
    assert!(!has_odbc_3_behavior!(odbc_2_stmt_handle));
    assert!(has_odbc_3_behavior!(odbc_3_env_handle));
    assert!(has_odbc_3_behavior!(odbc_3_conn_handle));
    assert!(has_odbc_3_behavior!(odbc_3_desc_handle));
    assert!(has_odbc_3_behavior!(odbc_3_stmt_handle));
}
