use odbc_sys::{
    AttrOdbcVersion, DriverConnectOption, EnvironmentAttribute, HDbc, HEnv, HStmt, Handle,
    HandleType, SQLAllocHandle, SQLDriverConnectW, SQLGetDiagRecW, SQLSetEnvAttr, SmallInt,
    SqlReturn, NTS,
};
use std::ptr::null_mut;
use std::{env, slice};
use thiserror::Error;
use cstr::{self, WideChar};
use constants::DRIVER_NAME;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("failed to allocate handle, SqlReturn: {0}")]
    HandleAllocation(String),
    #[error("failed to set environment attribute, SqlReturn: {0}")]
    SetEnvAttr(String),
    #[error("failed to connect SqlReturn: {0}, Diagnostics{1}")]
    DriverConnect(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Generate the default connection setting defined for the tests using a connection string
/// of the form 'Driver={};PWD={};USER={};SERVER={}'.
/// The default driver is 'MongoDB Atlas SQL ODBC Driver' if not specified.
/// The default auth db is 'admin' if not specified.
pub fn generate_default_connection_str() -> String {
    let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
    let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
    let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");

    let db = env::var("ADF_TEST_LOCAL_DB");
    let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
        Ok(val) => val,
        Err(_e) => DRIVER_NAME.to_string(), //Default driver name
    };

    let mut connection_string =
        format!("Driver={{{driver}}};USER={user_name};PWD={password};SERVER={host};");

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val + ";")),
        Err(_e) => (), // Do nothing
    };

    connection_string
}

#[allow(dead_code)]
// Verifies that the expected SQL State, message text, and native error in the handle match
// the expected input
pub fn get_sql_diagnostics(handle_type: HandleType, handle: Handle) -> String {
    let text_length_ptr = &mut 0;
    let mut actual_sql_state: [WideChar; 6] = [0; 6];
    let actual_sql_state = &mut actual_sql_state as *mut _;
    let mut actual_message_text: [WideChar; 512] = [0; 512];
    let actual_message_text = &mut actual_message_text as *mut _;
    let actual_native_error = &mut 0;
    unsafe {
        let _ = SQLGetDiagRecW(
            handle_type,
            handle as *mut _,
            1,
            actual_sql_state,
            actual_native_error,
            actual_message_text,
            1024,
            text_length_ptr,
        );
    };
    unsafe {
        cstr::from_widechar_ref_lossy(slice::from_raw_parts(
            actual_message_text as *const WideChar,
            *text_length_ptr as usize,
        ))
    }
}

#[allow(dead_code)]
/// Returns a String representation of the error code
pub fn sql_return_to_string(return_code: SqlReturn) -> String {
    match return_code {
        SqlReturn::SUCCESS => "SUCCESS".to_string(),
        SqlReturn::SUCCESS_WITH_INFO => "SUCCESS_WITH_INFO".to_string(),
        SqlReturn::NO_DATA => "NO_DATA".to_string(),
        SqlReturn::ERROR => "ERROR".to_string(),
        _ => {
            format!("{return_code:?}")
        }
    }
}

#[allow(dead_code)]
// Allocates new environment variable
pub fn allocate_env() -> Result<HEnv> {
    let mut env: Handle = null_mut();

    unsafe {
        match SQLAllocHandle(HandleType::Env, null_mut(), &mut env as *mut Handle) {
            SqlReturn::SUCCESS => (),
            sql_return => return Err(Error::HandleAllocation(sql_return_to_string(sql_return))),
        }
        match SQLSetEnvAttr(
            env as HEnv,
            EnvironmentAttribute::OdbcVersion,
            AttrOdbcVersion::Odbc3.into(),
            0,
        ) {
            SqlReturn::SUCCESS => (),
            sql_return => return Err(Error::SetEnvAttr(sql_return_to_string(sql_return))),
        }
    }
    Ok(env as HEnv)
}

#[allow(dead_code)]
// Connects to database with provided connection string
pub fn connect_with_conn_string(env_handle: HEnv, in_connection_string: String) -> Result<HDbc> {
    // Allocate a DBC handle
    let mut dbc: Handle = null_mut();
    unsafe {
        match SQLAllocHandle(
            HandleType::Dbc,
            env_handle as *mut _,
            &mut dbc as *mut Handle,
        ) {
            SqlReturn::SUCCESS => (),
            sql_return => return Err(Error::HandleAllocation(sql_return_to_string(sql_return))),
        }
        let mut in_connection_string_encoded = cstr::to_widechar_vec(&in_connection_string);
        in_connection_string_encoded.push(0);
        let str_len_ptr = &mut 0;
        match SQLDriverConnectW(
            dbc as HDbc,
            null_mut(),
            in_connection_string_encoded.as_ptr(),
            NTS as SmallInt,
            null_mut(),
            0,
            str_len_ptr,
            DriverConnectOption::NoPrompt,
        ) {
            SqlReturn::SUCCESS | SqlReturn::SUCCESS_WITH_INFO => (),
            sql_return => {
                return Err(Error::DriverConnect(
                    sql_return_to_string(sql_return),
                    get_sql_diagnostics(HandleType::Dbc, dbc),
                ))
            }
        }
    }
    Ok(dbc as HDbc)
}

#[allow(dead_code)]
// Allocate statement from connection handle
pub fn allocate_statement(dbc: HDbc) -> Result<HStmt> {
    let mut stmt: Handle = null_mut();
    unsafe {
        match SQLAllocHandle(HandleType::Stmt, dbc as *mut _, &mut stmt as *mut Handle) {
            SqlReturn::SUCCESS => (),
            sql_return => return Err(Error::HandleAllocation(sql_return_to_string(sql_return))),
        }
    }
    Ok(stmt as HStmt)
}
