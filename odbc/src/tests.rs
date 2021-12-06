use crate::{
    api::UNIMPLEMENTED_FUNC,
    handles::{
        Connection, ConnectionState, Descriptor, Env, EnvState, MongoHandle, Statement, StatementState
    },
    SQLAllocHandle, SQLFreeHandle,
    util::set_handle_state
};
use odbc_sys::{Handle, HandleType, SqlReturn, SQLGetDiagRec, Char, SmallInt, Integer};
use std::{sync::RwLock, ptr};

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

struct Buffers {
    sql_state: *mut Char,
    message_text: *mut Char,
    text_length_ptr: *mut i16,
    native_err_ptr: *mut i32
}

fn initialize_buffers() -> Buffers {
    unsafe {
        Buffers {
            sql_state: std::mem::transmute::<*mut u8, *mut Char>([0u8].as_mut_ptr()),
            message_text: std::mem::transmute::<*mut u8, *mut Char>([0u8].as_mut_ptr()),
            text_length_ptr: Box::into_raw( Box::new(0)),
            native_err_ptr: Box::into_raw( Box::new(0))
        }
    }
}

#[test]
fn set_sql_state() {
    let error_message = "func is unimplemented";
    unsafe {
        // Environment handle
        let env_handle: *mut _ =
          &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        let buffers = initialize_buffers();
        // TODO: native err ptr
        assert_eq!(Ok(()), set_handle_state(HandleType::Env, env_handle as *mut _, UNIMPLEMENTED_FUNC, error_message));
        assert_eq!(SqlReturn::SUCCESS, SQLGetDiagRec(HandleType::Env, env_handle as *mut _, 1, buffers.sql_state, buffers.native_err_ptr, buffers.message_text, 100, buffers.text_length_ptr));
        assert_eq!(UNIMPLEMENTED_FUNC.to_string(), (*env_handle).as_env().unwrap().read().unwrap().sql_states[0]);
        assert_eq!(error_message, (*env_handle).as_env().unwrap().read().unwrap().error_messages[0]);

        // Connection handle
        let conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(
            Connection::with_state(env_handle, ConnectionState::Allocated)
        ));
        let buffers = initialize_buffers();
        assert_eq!(Ok(()), set_handle_state(HandleType::Dbc, conn_handle as *mut _, UNIMPLEMENTED_FUNC, error_message));
        assert_eq!(SqlReturn::SUCCESS, SQLGetDiagRec(HandleType::Dbc, conn_handle as *mut _, 1, buffers.sql_state, buffers.native_err_ptr, buffers.message_text, 0, buffers.text_length_ptr));
        assert_eq!(UNIMPLEMENTED_FUNC.to_string(), (*conn_handle).as_connection().unwrap().read().unwrap().sql_states[0]);
        assert_eq!(error_message, (*conn_handle).as_connection().unwrap().read().unwrap().error_messages[0]);

        // Statement handle
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated
        )));
        let buffers = initialize_buffers();
        assert_eq!(Ok(()), set_handle_state(HandleType::Stmt, stmt_handle as *mut _, UNIMPLEMENTED_FUNC, error_message));
        assert_eq!(SqlReturn::SUCCESS, SQLGetDiagRec(HandleType::Stmt, stmt_handle as *mut _, 1, buffers.sql_state, buffers.native_err_ptr, buffers.message_text, 0, buffers.text_length_ptr));
        assert_eq!(UNIMPLEMENTED_FUNC.to_string(), (*stmt_handle).as_statement().unwrap().read().unwrap().sql_states[0]);
        assert_eq!(error_message, (*stmt_handle).as_statement().unwrap().read().unwrap().error_messages[0]);

        // Descriptor handle
        let desc_handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::default()));
        let buffers = initialize_buffers();
        assert_eq!(Ok(()), set_handle_state(HandleType::Desc, desc_handle as *mut _, UNIMPLEMENTED_FUNC, error_message));
        assert_eq!(SqlReturn::SUCCESS, SQLGetDiagRec(HandleType::Desc, desc_handle as *mut _, 1, buffers.sql_state, buffers.native_err_ptr, buffers.message_text, 0, buffers.text_length_ptr));
        assert_eq!(UNIMPLEMENTED_FUNC.to_string(), (*desc_handle).as_descriptor().unwrap().read().unwrap().sql_states[0]);
        assert_eq!(error_message, (*desc_handle).as_descriptor().unwrap().read().unwrap().error_messages[0]);
    }
}
