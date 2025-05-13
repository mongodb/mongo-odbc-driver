
use crate::{
    conn::MongoConnection,
    err::{Error, Result},
    odbc_uri::UserOptions,
    query::MongoQuery,
    MongoStatement,
    TypeMode,
};
use std::{ffi::{c_char, CStr, CString}, ptr, sync::Arc, time::Duration};
use tokio::runtime::Runtime;

#[repr(C)]
pub enum MongoOdbcErrorCode {
    Success = 0,
    ConnectionFailed = 1,
    QueryPreparationFailed = 2,
    QueryExecutionFailed = 3,
    InvalidParameter = 4,
    InvalidCursorState = 5,
    OutOfMemory = 6,
    UnknownError = 7,
}

#[repr(C)]
pub struct ConnectionHandle {
    connection: *mut MongoConnection,
}

#[repr(C)]
pub struct StatementHandle {
    statement: *mut Box<dyn MongoStatement>,
}

fn error_to_code(error: &Error) -> MongoOdbcErrorCode {
    match error {
        Error::InvalidClientOptions(_) => MongoOdbcErrorCode::ConnectionFailed,
        Error::InvalidUriFormat(_) => MongoOdbcErrorCode::InvalidParameter,
        Error::QueryExecutionFailed(_) => MongoOdbcErrorCode::QueryExecutionFailed,
        Error::InvalidCursorState => MongoOdbcErrorCode::InvalidCursorState,
        _ => MongoOdbcErrorCode::UnknownError,
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_connect(
    connection_string: *const c_char,
    error_code: *mut MongoOdbcErrorCode,
) -> *mut ConnectionHandle {
    if !error_code.is_null() {
        *error_code = MongoOdbcErrorCode::Success;
    }
    
    if connection_string.is_null() {
        if !error_code.is_null() {
            *error_code = MongoOdbcErrorCode::InvalidParameter;
        }
        return ptr::null_mut();
    }
    
    let conn_str = match CStr::from_ptr(connection_string).to_str() {
        Ok(s) => s,
        Err(_) => {
            if !error_code.is_null() {
                *error_code = MongoOdbcErrorCode::InvalidParameter;
            }
            return ptr::null_mut();
        }
    };
    
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => {
            if !error_code.is_null() {
                *error_code = MongoOdbcErrorCode::UnknownError;
            }
            return ptr::null_mut();
        }
    };
    
    let result = runtime.block_on(async {
        let mut user_options = match crate::odbc_uri::ODBCUri::new(conn_str.to_string()) {
            Ok(uri) => uri,
            Err(e) => {
                if !error_code.is_null() {
                    *error_code = error_to_code(&e);
                }
                return Err(e);
            }
        };
        
        let user_options = match user_options.try_into_client_options().await {
            Ok(opts) => opts,
            Err(e) => {
                if !error_code.is_null() {
                    *error_code = error_to_code(&e);
                }
                return Err(e);
            }
        };
        
        let db = None; // In a real implementation, we would parse this from the connection string
        
        match MongoConnection::connect(
            user_options,
            db,
            Some(30), // Default operation timeout
            Some(15), // Default login timeout
            TypeMode::Standard,
            Some(Runtime::new().unwrap()),
            None, // Default max string length
        ) {
            Ok(conn) => Ok(conn),
            Err(e) => {
                if !error_code.is_null() {
                    *error_code = error_to_code(&e);
                }
                Err(e)
            }
        }
    });
    
    match result {
        Ok(connection) => {
            let handle = Box::new(ConnectionHandle {
                connection: Box::into_raw(Box::new(connection)),
            });
            Box::into_raw(handle)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_free_connection(handle: *mut ConnectionHandle) {
    if !handle.is_null() {
        let handle = Box::from_raw(handle);
        if !handle.connection.is_null() {
            let connection = Box::from_raw(handle.connection);
            let _ = connection.shutdown();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_prepare_query(
    connection_handle: *const ConnectionHandle,
    query: *const c_char,
    error_code: *mut MongoOdbcErrorCode,
) -> *mut StatementHandle {
    if !error_code.is_null() {
        *error_code = MongoOdbcErrorCode::Success;
    }
    
    if connection_handle.is_null() || query.is_null() {
        if !error_code.is_null() {
            *error_code = MongoOdbcErrorCode::InvalidParameter;
        }
        return ptr::null_mut();
    }
    
    let query_str = match CStr::from_ptr(query).to_str() {
        Ok(s) => s,
        Err(_) => {
            if !error_code.is_null() {
                *error_code = MongoOdbcErrorCode::InvalidParameter;
            }
            return ptr::null_mut();
        }
    };
    
    let connection = &*(*connection_handle).connection;
    
    match MongoQuery::prepare(
        connection,
        None, // Use default database
        None, // No timeout
        query_str,
        TypeMode::Standard,
        None, // Default max string length
    ) {
        Ok(query) => {
            let statement_box: Box<dyn MongoStatement> = Box::new(query);
            let handle = Box::new(StatementHandle {
                statement: Box::into_raw(Box::new(statement_box)),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            if !error_code.is_null() {
                *error_code = error_to_code(&e);
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_execute_statement(
    connection_handle: *const ConnectionHandle,
    statement_handle: *mut StatementHandle,
    error_code: *mut MongoOdbcErrorCode,
) -> bool {
    if !error_code.is_null() {
        *error_code = MongoOdbcErrorCode::Success;
    }
    
    if connection_handle.is_null() || statement_handle.is_null() {
        if !error_code.is_null() {
            *error_code = MongoOdbcErrorCode::InvalidParameter;
        }
        return false;
    }
    
    let connection = &*(*connection_handle).connection;
    let statement = &mut *(*statement_handle).statement;
    
    match statement.execute(connection, mongodb::bson::Bson::Null, 100) {
        Ok(success) => success,
        Err(e) => {
            if !error_code.is_null() {
                *error_code = error_to_code(&e);
            }
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_free_statement(handle: *mut StatementHandle) {
    if !handle.is_null() {
        let handle = Box::from_raw(handle);
        if !handle.statement.is_null() {
            let _ = Box::from_raw(handle.statement);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_fetch(
    statement_handle: *mut StatementHandle,
    error_code: *mut MongoOdbcErrorCode,
) -> bool {
    if !error_code.is_null() {
        *error_code = MongoOdbcErrorCode::Success;
    }
    
    if statement_handle.is_null() {
        if !error_code.is_null() {
            *error_code = MongoOdbcErrorCode::InvalidParameter;
        }
        return false;
    }
    
    let statement = &mut *(*statement_handle).statement;
    
    match statement.next(None) {
        Ok((has_row, _)) => has_row,
        Err(e) => {
            if !error_code.is_null() {
                *error_code = error_to_code(&e);
            }
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mongo_odbc_get_error_message(error_code: MongoOdbcErrorCode) -> *const c_char {
    let message = match error_code {
        MongoOdbcErrorCode::Success => "Success",
        MongoOdbcErrorCode::ConnectionFailed => "Connection failed",
        MongoOdbcErrorCode::QueryPreparationFailed => "Query preparation failed",
        MongoOdbcErrorCode::QueryExecutionFailed => "Query execution failed",
        MongoOdbcErrorCode::InvalidParameter => "Invalid parameter",
        MongoOdbcErrorCode::InvalidCursorState => "Invalid cursor state",
        MongoOdbcErrorCode::OutOfMemory => "Out of memory",
        MongoOdbcErrorCode::UnknownError => "Unknown error",
    };
    
    match CString::new(message) {
        Ok(c_str) => {
            c_str.into_raw()
        }
        Err(_) => ptr::null(),
    }
}
