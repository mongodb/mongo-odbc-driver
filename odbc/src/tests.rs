use crate::{
    api::UNIMPLEMENTED_FUNC,
    handles::{
        Connection, ConnectionState, Descriptor, Env, EnvState, MongoHandle, Statement,
        StatementState,
    },
    util::set_handle_state,
    SQLAllocHandle, SQLFreeHandle, SQLGetDiagRec,
};
use odbc_sys::{Char, Handle, HandleType, SqlReturn};
use std::sync::RwLock;

const ERROR_MESSAGE: &str = "func is unimplemented";
const ERROR_MESSAGE_NULL: &str = "func is unimplemented\0";
const UNIMPLEMENTED_FUNC_NULL: &str = "HYC00\0";

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
fn env_diag_rec() {
    unsafe {
        let env_handle: *mut _ =
            &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
        assert_eq!(
            Ok(()),
            set_handle_state(
                HandleType::Env,
                env_handle as *mut _,
                UNIMPLEMENTED_FUNC.to_string(),
                ERROR_MESSAGE.to_string(),
                3
            )
        );
        // Initialize buffers
        let sql_state: *mut Char = [0u8; 6].as_mut_ptr();
        let message_text: *mut Char = [0u8; 22].as_mut_ptr();
        let text_length_ptr = Box::into_raw(Box::new(0));
        let native_err_ptr = Box::into_raw(Box::new(0));

        // Buffer is large enough to hold the entire error message + null terminator
        // (length >= 22)
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(
                HandleType::Env,
                env_handle as *mut _,
                1,
                sql_state,
                native_err_ptr,
                message_text,
                22,
                text_length_ptr
            )
        );
        assert_eq!(
            Ok(UNIMPLEMENTED_FUNC_NULL),
            std::str::from_utf8(&*(sql_state as *const [u8; 6]))
        );
        assert_eq!(
            Ok(ERROR_MESSAGE_NULL),
            std::str::from_utf8(&*(message_text as *const [u8; 22]))
        );
        assert_eq!(22, *text_length_ptr);
        // Buffer is too small to hold the entire error message (0 < length < 22)
        assert_eq!(
            SqlReturn::SUCCESS_WITH_INFO,
            SQLGetDiagRec(
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
            Ok("func is unimpl\0"),
            std::str::from_utf8(&*(message_text as *const [u8; 15]))
        );
        assert_eq!(15, *text_length_ptr);
        // Buffer length < 0
        assert_eq!(
            SqlReturn::ERROR,
            SQLGetDiagRec(
                HandleType::Env,
                env_handle as *mut _,
                0,
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
            SQLGetDiagRec(
                HandleType::Env,
                env_handle as *mut _,
                0,
                sql_state,
                native_err_ptr,
                message_text,
                22,
                text_length_ptr
            )
        );
        // 1 < RecNumber =< number of diagnostic records
        assert_eq!(
            Ok(()),
            set_handle_state(
                HandleType::Env,
                env_handle as *mut _,
                "XYZ00".to_string(),
                ERROR_MESSAGE.to_string(),
                3
            )
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetDiagRec(
                HandleType::Env,
                env_handle as *mut _,
                2,
                sql_state,
                native_err_ptr,
                message_text,
                22,
                text_length_ptr
            )
        );
        assert_eq!(
            Ok("XYZ00\0"),
            std::str::from_utf8(&*(sql_state as *const [u8; 6]))
        );
        // Record number > number of diagnostic records
        assert_eq!(
            SqlReturn::NO_DATA,
            SQLGetDiagRec(
                HandleType::Env,
                env_handle as *mut _,
                3,
                sql_state,
                native_err_ptr,
                message_text,
                22,
                text_length_ptr
            )
        );
        // Native error pointer
        assert_eq!(3, *native_err_ptr);
    }
}

#[test]
fn conn_diag_rec() {
    let env_handle: *mut _ =
        &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
    let conn_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(Connection::with_state(
        env_handle,
        ConnectionState::Allocated,
    )));
    assert_eq!(
        Ok(()),
        set_handle_state(
            HandleType::Dbc,
            conn_handle as *mut _,
            UNIMPLEMENTED_FUNC.to_string(),
            ERROR_MESSAGE.to_string(),
            0
        )
    );

    // Initialize buffers
    let sql_state: *mut Char = [0u8; 6].as_mut_ptr();
    let message_text: *mut Char = [0u8; 22].as_mut_ptr();
    let text_length_ptr = Box::into_raw(Box::new(0));
    let native_err_ptr = Box::into_raw(Box::new(0));
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetDiagRec(
            HandleType::Dbc,
            conn_handle as *mut _,
            1,
            sql_state,
            native_err_ptr,
            message_text,
            50,
            text_length_ptr,
        )
    );
    assert_eq!(Ok(UNIMPLEMENTED_FUNC_NULL), unsafe {
        std::str::from_utf8(&*(sql_state as *const [u8; 6]))
    });
    assert_eq!(Ok(ERROR_MESSAGE_NULL), unsafe {
        std::str::from_utf8(&*(message_text as *const [u8; 22]))
    });
}

#[test]
fn stmt_diag_rec() {
    let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
        std::ptr::null_mut(),
        StatementState::Allocated,
    )));
    assert_eq!(
        Ok(()),
        set_handle_state(
            HandleType::Stmt,
            stmt_handle as *mut _,
            UNIMPLEMENTED_FUNC.to_string(),
            ERROR_MESSAGE.to_string(),
            0
        )
    );
    // Initialize buffers
    let sql_state: *mut Char = [0u8; 6].as_mut_ptr();
    let message_text: *mut Char = [0u8; 22].as_mut_ptr();
    let text_length_ptr = Box::into_raw(Box::new(0));
    let native_err_ptr = Box::into_raw(Box::new(0));
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetDiagRec(
            HandleType::Stmt,
            stmt_handle as *mut _,
            1,
            sql_state,
            native_err_ptr,
            message_text,
            50,
            text_length_ptr,
        )
    );
    assert_eq!(Ok(UNIMPLEMENTED_FUNC_NULL), unsafe {
        std::str::from_utf8(&*(sql_state as *const [u8; 6]))
    });
    assert_eq!(Ok(ERROR_MESSAGE_NULL), unsafe {
        std::str::from_utf8(&*(message_text as *const [u8; 22]))
    });
}

#[test]
fn desc_diag_rec() {
    let desc_handle: *mut _ = &mut MongoHandle::Descriptor(RwLock::new(Descriptor::default()));
    assert_eq!(
        Ok(()),
        set_handle_state(
            HandleType::Desc,
            desc_handle as *mut _,
            UNIMPLEMENTED_FUNC.to_string(),
            ERROR_MESSAGE.to_string(),
            0
        )
    );
    // Initialize buffers
    let sql_state: *mut Char = [0u8; 6].as_mut_ptr();
    let message_text: *mut Char = [0u8; 22].as_mut_ptr();
    let text_length_ptr = Box::into_raw(Box::new(0));
    let native_err_ptr = Box::into_raw(Box::new(0));
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetDiagRec(
            HandleType::Desc,
            desc_handle as *mut _,
            1,
            sql_state,
            native_err_ptr,
            message_text,
            50,
            text_length_ptr,
        )
    );
    assert_eq!(Ok(UNIMPLEMENTED_FUNC_NULL), unsafe {
        std::str::from_utf8(&*(sql_state as *const [u8; 6]))
    });
    assert_eq!(Ok(ERROR_MESSAGE_NULL), unsafe {
        std::str::from_utf8(&*(message_text as *const [u8; 22]))
    });
}
