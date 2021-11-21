use crate::{
    handles::{
        Connection, ConnectionState, Descriptor, DescriptorState, Env, EnvState, MongoHandle,
        Statement, StatementState,
    },
    SQLAllocHandle, SQLFreeHandle,
};
use odbc_sys::{Handle, HandleType, SqlReturn};
use std::sync::RwLock;

#[test]
fn env_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Env(RwLock::new(Env::new()));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            EnvState::Unallocated,
            (*handle).as_env().unwrap().read().unwrap().state
        );
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
fn desc_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::new()));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            DescriptorState::Unallocated,
            (*handle).as_descriptor().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Desc,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            DescriptorState::ExplicitlyAllocated,
            (*handle).as_descriptor().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Desc,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn connection_alloc_free() {
    unsafe {
        let env_handle: *mut _ = &mut MongoHandle::Env(RwLock::new(Env::new()));
        (*env_handle).as_env().unwrap().write().unwrap().state = EnvState::Allocated;

        let mut handle: *mut _ =
            &mut MongoHandle::Connection(RwLock::new(Connection::new(std::ptr::null_mut())));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            ConnectionState::AllocatedEnvUnallocatedConnection,
            (*handle).as_connection().unwrap().read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::AllocatedEnvAllocatedConnection,
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
        let env_handle: *mut _ = &mut MongoHandle::Env(RwLock::new(Env::new()));
        (*env_handle).as_env().unwrap().write().unwrap().state = EnvState::Allocated;

        let conn_handle: *mut _ =
            &mut MongoHandle::Connection(RwLock::new(Connection::new(env_handle)));
        (*conn_handle)
            .as_connection()
            .unwrap()
            .write()
            .unwrap()
            .state = ConnectionState::AllocatedEnvAllocatedConnection;

        let mut handle: *mut _ =
            &mut MongoHandle::Statement(RwLock::new(Statement::new(std::ptr::null_mut())));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            StatementState::Unallocated,
            (*handle).as_statement().unwrap().read().unwrap().state
        );
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
        let mut handle: *mut _ = &mut MongoHandle::Env(RwLock::new(Env::new()));
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            EnvState::Unallocated,
            (*handle).as_env().unwrap().read().unwrap().state
        );
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
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLFreeHandle(
                HandleType::Desc,
                std::mem::transmute::<*mut MongoHandle, Handle>(handle),
            )
        );
        // Free for real so we don't leak.
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
fn invalid_alloc() {
    unsafe {
        let mut handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::new()));
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

        let desc_handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::new()));

        // now test wrong handle type
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Dbc,
                desc_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            SqlReturn::INVALID_HANDLE,
            SQLAllocHandle(
                HandleType::Stmt,
                desc_handle as *mut _,
                std::mem::transmute::<*mut *mut MongoHandle, *mut Handle>(handle_ptr),
            )
        );
    }
}
