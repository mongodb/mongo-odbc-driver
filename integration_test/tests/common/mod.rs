use constants::DRIVER_NAME;
use cstr::{self, WideChar};
use definitions::{
    AttrOdbcVersion, CDataType, Desc, DriverConnectOption, EnvironmentAttribute, HDbc, HEnv, HStmt,
    Handle, HandleType, Len, Pointer, SQLAllocHandle, SQLColAttributeW, SQLDisconnect,
    SQLDriverConnectW, SQLFetch, SQLFreeHandle, SQLGetData, SQLGetDiagRecW, SQLMoreResults,
    SQLNumResultCols, SQLSetEnvAttr, SmallInt, SqlReturn, USmallInt, NTS,
};
use std::ptr::null_mut;
use std::{env, slice};
use thiserror::Error;

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

pub const BUFFER_LENGTH: SmallInt = 300;

pub struct OutputBuffer {
    pub output_buffer: Pointer,
    pub data_length: i16,
}

impl From<OutputBuffer> for String {
    fn from(val: OutputBuffer) -> Self {
        unsafe {
            cstr::from_widechar_ref_lossy(slice::from_raw_parts(
                val.output_buffer as *const _,
                val.data_length as usize / std::mem::size_of::<WideChar>(),
            ))
        }
    }
}

impl From<OutputBuffer> for u16 {
    fn from(val: OutputBuffer) -> Self {
        unsafe { *(val.output_buffer as *mut u16) }
    }
}

#[allow(dead_code)] // false positive
/// Generate a default uri
pub fn generate_uri_with_default_connection_string(uri: &str) -> String {
    let host_port = env::var("ADF_TEST_URI").expect("ADF_TEST_URI is not set");
    let combined_uri = format!("{}/?{}", host_port, uri);
    format!("{}URI={combined_uri}", generate_default_connection_str())
}

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
            EnvironmentAttribute::SQL_ATTR_ODBC_VERSION,
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
/// Connect to a database and allocate a new statement
/// Return the db and statement handles back for future use.
pub fn connect_and_allocate_statement(
    env_handle: HEnv,
    in_connection_string: Option<String>,
) -> (HDbc, HStmt) {
    let conn_str = match in_connection_string {
        Some(val) => val,
        None => crate::common::generate_default_connection_str(),
    };
    let conn_handle = connect_with_conn_string(env_handle, conn_str).unwrap();
    (conn_handle, allocate_statement(conn_handle).unwrap())
}

#[allow(dead_code)]
/// Connects to database with provided connection string
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
            // Originally, this would return SUCCESS_WITH_INFO since we pass null_mut() as
            // out_connection_string and 0 as buffer size. Now, this should always return SUCCESS.
            SqlReturn::SUCCESS => (),
            // TODO SQL-1568: Windows DM is still changing SUCCESS to SUCCESS_WITH_INFO
            SqlReturn::SUCCESS_WITH_INFO => {
                if !cfg!(windows) {
                    return Err(Error::DriverConnect(
                        sql_return_to_string(SqlReturn::SUCCESS_WITH_INFO),
                        get_sql_diagnostics(HandleType::Dbc, dbc),
                    ));
                }
            }
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
/// Allocate statement from connection handle
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

#[allow(dead_code)]
///  Helper function for freeing handles and closing the connection
///  - SQLFreeHandle(stmt)
///  - SQLDisconnect(dbc)
///  - SQLFreeHandle(dbc)
pub fn disconnect_and_close_handles(dbc: HDbc, stmt: HStmt) {
    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::Stmt, stmt as Handle),
            "{}",
            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLDisconnect(dbc),
            "{}",
            get_sql_diagnostics(HandleType::Dbc, dbc as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::Dbc, dbc as Handle),
            "{}",
            get_sql_diagnostics(HandleType::Stmt, dbc as Handle)
        );
    }
}

#[allow(dead_code)]
///  Helper function for fetching and getting data
///  - Until SQLFetch returns SQL_NO_DATA
///      - SQLFetch()
///      - For columns 1 to {numCols}
///          - SQLGetData({colIndex}, {defaultCtoSqlType})
///  - SQLMoreResults()
pub fn fetch_and_get_data(
    stmt: Handle,
    expected_fetch_count: Option<SmallInt>,
    expected_sql_returns: Vec<SqlReturn>,
    target_types: Vec<CDataType>,
) {
    let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;
    let mut successful_fetch_count = 0;
    let str_len_ptr = &mut 0;
    unsafe {
        loop {
            let result = SQLFetch(stmt as HStmt);
            assert!(
                result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA,
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
            match result {
                SqlReturn::SUCCESS => {
                    successful_fetch_count += 1;
                    for col_num in 0..target_types.len() {
                        assert_eq!(
                            expected_sql_returns[col_num],
                            SQLGetData(
                                stmt as HStmt,
                                (col_num + 1) as USmallInt,
                                target_types[col_num],
                                output_buffer as Pointer,
                                (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                                str_len_ptr
                            ),
                            "{}",
                            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                        );
                    }
                }
                // break if SQLFetch returns SQL_NO_DATA
                _ => break,
            }
        }

        if let Some(exp_fetch_count) = expected_fetch_count {
            assert_eq!(
                exp_fetch_count, successful_fetch_count,
                "Expected {exp_fetch_count:?} successful calls to SQLFetch, got {successful_fetch_count}."
            );
        }

        assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));
    }
}

#[allow(dead_code)]
///  Helper function for checking resultset metadata
///  - SQLNumResultCols()
///  - For columns 1 to {numCols}
///      - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
///      - SQLColAttributeW(SQL_DESC_UNSIGNED)
///      - SQLColAttributeW(SQL_COLUMN_NAME)
///      - SQLColAttributeW(SQL_COLUMN_NULLABLE)
///      - SQLColAttributeW(SQL_DESC_TYPE_NAME)
///      - SQLColAttributeW(SQL_COLUMN_LENGTH)
///      - SQLColAttributeW(SQL_COLUMN_SCALE)
pub fn get_column_attributes(stmt: Handle, expected_col_count: SmallInt) {
    let str_len_ptr = &mut 0;
    let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;
    unsafe {
        let column_count_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLNumResultCols(stmt as HStmt, column_count_ptr),
            "{}",
            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
        );
        assert_eq!(expected_col_count, *column_count_ptr);

        let numeric_attribute_ptr = &mut 0;
        const FIELD_IDS: [Desc; 7] = [
            Desc::ConciseType,
            Desc::Unsigned,
            Desc::Name,
            Desc::Nullable,
            Desc::TypeName,
            Desc::Length,
            Desc::Scale,
        ];
        for col_num in 0..*column_count_ptr {
            FIELD_IDS.iter().for_each(|field_type| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLColAttributeW(
                        stmt as HStmt,
                        (col_num + 1) as u16,
                        *field_type,
                        output_buffer as Pointer,
                        BUFFER_LENGTH,
                        str_len_ptr,
                        numeric_attribute_ptr,
                    ),
                    "{}",
                    get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                );
            });
        }
    }
}
