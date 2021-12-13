use crate::{
    api::UNIMPLEMENTED_FUNC,
    handles::{
        Connection, ConnectionState, Descriptor, Env, EnvState, MongoHandle, Statement, StatementState
    },
    SQLAllocHandle, SQLFreeHandle,
    util::set_handle_state
};
use odbc_sys::{Handle, HandleType, SqlReturn, SQLGetDiagRec, Char};
use std::sync::RwLock;
use std::ffi::CString;
use std::io::Read;

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

#[test]
fn varsha() {
    unsafe {
        let output_ptr = std::mem::transmute::<*mut u8, *mut Char>([0u8;5].as_mut_ptr());
        let c_str = std::mem::transmute::<*mut u8, *mut Char>("VARSHA".to_string().as_mut_ptr());
        std::ptr::copy_nonoverlapping(c_str, output_ptr, 6);
        // println!("AAA: {:?}", (*output_ptr).to_string());
        let a = std::mem::transmute::<*mut u8, &[u8;6]>(output_ptr);
        println!("{:?}", std::str::from_utf8(a));
    }
}

#[test]
fn set_sql_state() {
    unsafe {
        // Environment handle
        let env_handle: *mut _ =
          &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        let error_message = "func is unimplemented";
        assert_eq!(
            Ok(()),
            set_handle_state(HandleType::Env, env_handle as *mut _, UNIMPLEMENTED_FUNC, error_message)
        );
        // Initialize buffers
        let sql_state: *mut Char = [0u8;5].as_mut_ptr();
        let message_text: *mut Char = [0u8;21].as_mut_ptr();
        let text_length_ptr = Box::into_raw( Box::new(0));
        let native_err_ptr = Box::into_raw( Box::new(0));
        // Buffer is large enough to hold the entire error message (length >= 21)
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(HandleType::Env, env_handle as *mut _, 1, sql_state, native_err_ptr, message_text, 50, text_length_ptr)
        );
        assert_eq!(
            Ok(UNIMPLEMENTED_FUNC.clone()),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;5]>(sql_state))
        );
        assert_eq!(
            Ok(error_message),
            // len(error_message) = 21
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;21]>(message_text))
        );
        // Buffer is too small to hold the entire error message (length < 21)
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(HandleType::Env, env_handle as *mut _, 1, sql_state, native_err_ptr, message_text, 15, text_length_ptr)
        );
        assert_eq!(
            Ok("func is unimple"),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;15]>(message_text))
        );

        // Connection handle
        let conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(
            Connection::with_state(env_handle, ConnectionState::Allocated)
        ));
        assert_eq!(
            Ok(()),
            set_handle_state(HandleType::Dbc, conn_handle as *mut _, UNIMPLEMENTED_FUNC, error_message)
        );

        // Reset buffers
        let sql_state: *mut Char = [0u8;5].as_mut_ptr();
        let message_text: *mut Char = [0u8;21].as_mut_ptr();
        let text_length_ptr = Box::into_raw( Box::new(0));
        let native_err_ptr = Box::into_raw( Box::new(0));
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(HandleType::Dbc, conn_handle as *mut _, 1, sql_state, native_err_ptr, message_text, 50, text_length_ptr)
        );
        assert_eq!(
            Ok(UNIMPLEMENTED_FUNC.clone()),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;5]>(sql_state))
        );
        assert_eq!(
            Ok(error_message),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;21]>(message_text))
        );

        // Statement handle
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated
        )));
        assert_eq!(
            Ok(()),
            set_handle_state(HandleType::Stmt, stmt_handle as *mut _, UNIMPLEMENTED_FUNC, error_message)
        );
        // Reset buffers
        let sql_state: *mut Char = [0u8;5].as_mut_ptr();
        let message_text: *mut Char = [0u8;21].as_mut_ptr();
        let text_length_ptr = Box::into_raw( Box::new(0));
        let native_err_ptr = Box::into_raw( Box::new(0));
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(HandleType::Stmt, stmt_handle as *mut _, 1, sql_state, native_err_ptr, message_text, 50, text_length_ptr)
        );
        assert_eq!(
            Ok(UNIMPLEMENTED_FUNC.clone()),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;5]>(sql_state))
        );
        assert_eq!(
            Ok(error_message),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;21]>(message_text))
        );

        // Descriptor handle
        let desc_handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::default()));
        assert_eq!(
            Ok(()),
            set_handle_state(HandleType::Desc, desc_handle as *mut _, UNIMPLEMENTED_FUNC, error_message)
        );
        // Reset buffers
        let sql_state: *mut Char = [0u8;5].as_mut_ptr();
        let message_text: *mut Char = [0u8;21].as_mut_ptr();
        let text_length_ptr = Box::into_raw( Box::new(0));
        let native_err_ptr = Box::into_raw( Box::new(0));
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(HandleType::Desc, desc_handle as *mut _, 1, sql_state, native_err_ptr, message_text, 50, text_length_ptr)
        );
        assert_eq!(
            Ok(UNIMPLEMENTED_FUNC.clone()),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;5]>(sql_state))
        );
        assert_eq!(
            Ok(error_message),
            std::str::from_utf8(std::mem::transmute::<*mut u8, &[u8;21]>(message_text))
        );
    }
}
