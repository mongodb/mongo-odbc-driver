use crate::{
    handles::{
        Connection, ConnectionHandle, ConnectionState, Env, EnvHandle, EnvState, Statement,
        StatementHandle, StatementState,
    },
    SQLAllocHandle, SQLFreeHandle,
};
use odbc_sys::{Handle, HandleType, SqlReturn};
use std::sync::RwLock;

#[test]
fn env_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut RwLock::new(Env::new());
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(EnvState::Unallocated, (*handle).read().unwrap().state);
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Env,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut EnvHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(EnvState::Allocated, (*handle).read().unwrap().state);
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Env,
                std::mem::transmute::<*mut EnvHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn connection_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut RwLock::new(Env::new());
        (*env_handle).write().unwrap().state = EnvState::Allocated;

        let mut handle: *mut _ = &mut RwLock::new(Connection::new(std::ptr::null_mut()));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            ConnectionState::AllocatedEnvUnallocatedConnection,
            (*handle).read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut ConnectionHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::AllocatedEnvAllocatedConnection,
            (*handle).read().unwrap().state
        );
        assert_eq!(1, (*env_handle).read().unwrap().connections.len());
        assert_eq!(
            EnvState::ConnectionAllocated,
            (*env_handle).read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut ConnectionHandle, Handle>(handle),
            )
        );
        assert_eq!(0, (*env_handle).read().unwrap().connections.len());
        assert_eq!(EnvState::Allocated, (*env_handle).read().unwrap().state);
    }
}

#[test]
fn statement_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut RwLock::new(Env::new());
        (*env_handle).write().unwrap().state = EnvState::Allocated;

        let conn_handle: *mut _ = &mut RwLock::new(Connection::new(env_handle));
        (*conn_handle).write().unwrap().state = ConnectionState::AllocatedEnvAllocatedConnection;

        let mut handle: *mut _ = &mut RwLock::new(Statement::new(std::ptr::null_mut()));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            StatementState::Unallocated,
            (*handle).write().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Stmt,
                conn_handle as *mut _,
                std::mem::transmute::<*mut *mut StatementHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(StatementState::Allocated, (*handle).write().unwrap().state);
        assert_eq!(1, (*conn_handle).read().unwrap().statements.len());
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut StatementHandle, Handle>(handle),
            )
        );
        assert_eq!(0, (*conn_handle).read().unwrap().statements.len());
    }
}
