use crate::{handles::definitions::*, SQLAllocHandle, SQLFreeHandle};
use odbc_sys::{Handle, HandleType, SqlReturn};

#[test]
fn test_env_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
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
            *(*handle).as_env().unwrap().state.read().unwrap(),
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
                HandleType::Dbc,
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
                HandleType::Dbc,
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
                HandleType::Stmt,
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
                HandleType::Stmt,
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
fn test_invalid_free() {
    unsafe {
        let mut env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
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
            *(*env_handle).as_env().unwrap().state.read().unwrap(),
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

        let mut conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
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
fn test_invalid_alloc() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
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

        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

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
