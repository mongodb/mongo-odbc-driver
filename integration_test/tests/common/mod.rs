#![allow(
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
use constants::DRIVER_NAME;
use cstr::{self, WideChar};
use definitions::{
    AttrOdbcVersion, CDataType, Desc, DriverConnectOption, EnvironmentAttribute, HDbc, HEnv, HStmt,
    Handle, HandleType, Len, Pointer, SQLAllocHandle, SQLBindCol, SQLColAttributeW, SQLDisconnect,
    SQLDriverConnectW, SQLExecDirectW, SQLFetch, SQLFreeHandle, SQLGetData, SQLGetDiagRecW,
    SQLMoreResults, SQLNumResultCols, SQLSetEnvAttr, SmallInt, SqlReturn, USmallInt, SQL_NTS,
};
use serde_json::{json, Value};
use std::ptr::null_mut;
use std::{env, slice};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("failed to allocate handle, SqlReturn: {0}")]
    HandleAllocation(String),
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
    let combined_uri = format!("{host_port}/?{uri}");
    format!("{}URI={combined_uri}", generate_default_connection_str())
}

/// Generate
/// of the form 'Driver={};PWD={};USER={};SERVER={}'.
/// The default driver is 'MongoDB Atlas SQL ODBC Driver' if not specified.
/// The default auth db is 'admin' if not specified.
pub fn generate_default_connection_str() -> String {
    generate_connection_str(None, None)
}

/// Generate the a connection setting defined for the tests using a connection string with an
/// optional user name
/// of the form 'Driver={};PWD={};USER={};SERVER={}'.
/// The default driver is 'MongoDB Atlas SQL ODBC Driver' if not specified.
/// The default auth db is 'admin' if not specified.
pub fn generate_connection_str(user: Option<String>, database: Option<String>) -> String {
    let user_name = if let Some(user) = user {
        user
    } else {
        env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set")
    };
    let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
    let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");

    let db = if let Some(db) = database {
        Ok(db)
    } else {
        env::var("ADF_TEST_LOCAL_DB")
    };
    let driver = env::var("ADF_TEST_LOCAL_DRIVER").unwrap_or_else(|_e| DRIVER_NAME.to_string());

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
/// generate a "mongodb+srv" connection string based on the specified environmental variables.
pub fn generate_srv_style_connection_string(db: Option<String>) -> String {
    // The driver used is the same as the one used for ADF
    let driver = env::var("ADF_TEST_LOCAL_DRIVER").unwrap_or_else(|_e| DRIVER_NAME.to_string());

    let db = if let Some(db) = db {
        db
    } else {
        env::var("SRV_TEST_DB").expect("SRV_TEST_DB is not set")
    };

    let auth_db = env::var("SRV_TEST_AUTH_DB").expect("SRV_TEST_AUTH_DB is not set");
    let host = env::var("SRV_TEST_HOST").expect("SRV_TEST_HOST is not set");
    let username = env::var("SRV_TEST_USER").expect("SRV_TEST_USER is not set");
    let password = env::var("SRV_TEST_PWD").expect("SRV_TEST_PWD is not set");

    let mongodb_uri = format!("mongodb+srv://{username}:{password}@{host}/?authSource={auth_db}");

    format!("DRIVER={driver};DATABASE={db};URI={mongodb_uri}")
}

#[allow(dead_code)]
pub struct SqlDiagnostics {
    pub sqlstate: String,
    pub error_message: String,
}
// Returns the SQL State and message text for the error from the input handle
pub fn get_sql_diagnostics_full(handle_type: HandleType, handle: Handle) -> SqlDiagnostics {
    let text_length_ptr = &mut 0;
    let mut actual_sql_state: [WideChar; 6] = [0; 6];
    let actual_sql_state = &mut actual_sql_state as *mut _;
    let mut actual_message_text: [WideChar; 512] = [0; 512];
    let actual_message_text = &mut actual_message_text as *mut _;
    let actual_native_error = &mut 0;
    unsafe {
        let _ = SQLGetDiagRecW(
            handle_type as i16,
            handle as *mut _,
            1,
            actual_sql_state,
            actual_native_error,
            actual_message_text,
            1024,
            text_length_ptr,
        );
    };
    let sqlstate = unsafe {
            cstr::from_widechar_ref_lossy(slice::from_raw_parts(
                actual_sql_state as *const WideChar,
                5,
        ))
    };
    let message = unsafe {
            cstr::from_widechar_ref_lossy(slice::from_raw_parts(
                actual_message_text as *const WideChar,
                *text_length_ptr as usize,
        ))
    };
    SqlDiagnostics { sqlstate, error_message: message }
}

// Returns the message text from the error in the input handle
pub fn get_sql_diagnostics(handle_type: HandleType, handle: Handle) -> String {
    get_sql_diagnostics_full(handle_type, handle).error_message
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
/// Setup flow that connects and allocates a statement. This allocates a new
/// environment handle, sets the ODBC_VERSION environment attribute, connects
/// using the provided (or default) URI, and allocates a statement. The flow
/// is:
///     - SQLAllocHandle(SQL_HANDLE_ENV)
///     - SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
///     - SQLAllocHandle(SQL_HANDLE_DBC)
///     - SQLDriverConnectW
///     - SQLAllocHandle(SQL_HANDLE_STMT)
pub fn default_setup_connect_and_alloc_stmt(
    odbc_version_value: AttrOdbcVersion,
    in_connection_string: Option<String>,
) -> (HEnv, HDbc, HStmt) {
    let env_handle = allocate_env(odbc_version_value);
    let (conn_handle, stmt_handle) =
        connect_and_allocate_statement(env_handle, in_connection_string);

    (env_handle, conn_handle, stmt_handle)
}

#[allow(dead_code)]
/// Allocates new environment variable and sets the ODBC_VERSION environment
/// attribute to the provided odbc_version.
pub fn allocate_env(odbc_version: AttrOdbcVersion) -> HEnv {
    let mut env: Handle = null_mut();

    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::SQL_HANDLE_ENV as i16,
                null_mut(),
                &mut env as *mut Handle,
            ),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env as Handle)
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::SQL_ATTR_ODBC_VERSION as i32,
                odbc_version.into(),
                0
            ),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env as Handle)
        );
    }
    env as HEnv
}

#[allow(dead_code)]
/// Connect to a database and allocate a new statement
/// Return the db and statement handles back for future use.
pub fn connect_and_allocate_statement(
    env_handle: HEnv,
    in_connection_string: Option<String>,
) -> (HDbc, HStmt) {
    let conn_handle = connect_with_conn_string(env_handle, in_connection_string, true).unwrap();
    (conn_handle, allocate_statement(conn_handle).unwrap())
}

#[allow(dead_code)]
/// Connects to database with provided connection string, or default connection string
pub fn connect_with_conn_string(
    env_handle: HEnv,
    in_connection_string: Option<String>,
    use_str_len_ptr: bool,
) -> Result<HDbc> {
    // Allocate a DBC handle
    let mut dbc: Handle = null_mut();
    unsafe {
        match SQLAllocHandle(
            HandleType::SQL_HANDLE_DBC as i16,
            env_handle as *mut _,
            &mut dbc as *mut Handle,
        ) {
            SqlReturn::SUCCESS => (),
            sql_return => return Err(Error::HandleAllocation(sql_return_to_string(sql_return))),
        }
        let in_connection_string =
            in_connection_string.unwrap_or_else(generate_default_connection_str);
        println!("Connecting with connection string: {in_connection_string}");
        let mut in_connection_string_encoded = cstr::to_widechar_vec(&in_connection_string);
        in_connection_string_encoded.push(0);
        let mut len_buffer: i16 = 0;
        let str_len_ptr = if use_str_len_ptr {
            &mut len_buffer as *mut _
        } else {
            null_mut()
        };
        match SQLDriverConnectW(
            dbc as HDbc,
            null_mut(),
            in_connection_string_encoded.as_ptr(),
            SQL_NTS.try_into().unwrap(),
            null_mut(),
            0,
            str_len_ptr,
            DriverConnectOption::SQL_DRIVER_NO_PROMPT as u16,
        ) {
            // Originally, this would return SUCCESS_WITH_INFO since we pass null_mut() as
            // out_connection_string and 0 as buffer size. Now, this should always return SUCCESS.
            SqlReturn::SUCCESS => (),
            // TODO SQL-1568: Windows DM is still changing SUCCESS to SUCCESS_WITH_INFO
            SqlReturn::SUCCESS_WITH_INFO => {
                if !cfg!(windows) {
                    return Err(Error::DriverConnect(
                        sql_return_to_string(SqlReturn::SUCCESS_WITH_INFO),
                        get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, dbc),
                    ));
                }
            }
            sql_return => {
                return Err(Error::DriverConnect(
                    sql_return_to_string(sql_return),
                    get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, dbc),
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
        match SQLAllocHandle(
            HandleType::SQL_HANDLE_STMT as i16,
            dbc as *mut _,
            &mut stmt as *mut Handle,
        ) {
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
            SQLFreeHandle(HandleType::SQL_HANDLE_STMT as i16, stmt as Handle),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLDisconnect(dbc),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, dbc as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_DBC as i16, dbc as Handle),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, dbc as Handle)
        );
    }
}

#[allow(dead_code)]
/// Helper function for disconnecting and freeing HDbc and HEnv handles.
/// Note that this function explicitly does NOT free the statement handle
/// since it is intended for use with tests that invoke SQLFreeStmt.
///  - SQLDisconnect(dbc)
///  - SQLFreeHandle(dbc)
///  - SQLFreeHandle(env)
pub fn disconnect_and_free_dbc_and_env_handles(env_handle: HEnv, conn_handle: HDbc) {
    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLDisconnect(conn_handle),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, conn_handle as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_DBC as i16, conn_handle as Handle),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, conn_handle as Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(HandleType::SQL_HANDLE_ENV as i16, env_handle as Handle),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env_handle as Handle)
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
    mongosqltranslate_test_exp_vals: Option<Vec<Vec<Value>>>,
) {
    let output_buffer: *mut std::ffi::c_void =
        Box::into_raw(Box::new([0u8; BUFFER_LENGTH as usize])) as *mut _;
    let mut successful_fetch_count = 0;
    let str_len_ptr = &mut 0;

    unsafe {
        let mut loop_index = 0;
        loop {
            let result = SQLFetch(stmt as HStmt);
            assert!(
                result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA,
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
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
                                target_types[col_num] as i16,
                                output_buffer as Pointer,
                                (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                                str_len_ptr
                            ),
                            "{}",
                            get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
                        );

                        if let Some(ref values) = mongosqltranslate_test_exp_vals {
                            let actual_val = match target_types[col_num] {
                                CDataType::SQL_C_SLONG => json!(*(output_buffer as *mut i32)),
                                CDataType::SQL_C_WCHAR => {
                                    if *str_len_ptr < 0 {
                                        Value::Null
                                    } else {
                                        json!(cstr::from_widechar_ref_lossy(
                                            &*(output_buffer
                                                as *const [WideChar; BUFFER_LENGTH as usize])
                                        )[0..(*str_len_ptr as usize
                                            / std::mem::size_of::<WideChar>())]
                                            .to_string())
                                    }
                                }
                                _ => unreachable!("An unexpected type was encountered."),
                            };

                            assert_eq!(actual_val, values[loop_index][col_num]);
                        }
                    }
                }
                // break if SQLFetch returns SQL_NO_DATA
                _ => break,
            }
            loop_index += 1;
        }

        if let Some(exp_fetch_count) = expected_fetch_count {
            assert_eq!(
                exp_fetch_count, successful_fetch_count,
                "Expected {exp_fetch_count:?} successful calls to SQLFetch, got {successful_fetch_count}."
            );
        }

        assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));

        let _ = Box::from_raw(output_buffer as *mut [u8; BUFFER_LENGTH as usize]);
    }
}

#[allow(dead_code)]
/// Helper function for checking resultset metadata
/// - SQLNumResultCols()
/// - For columns 1 to {numCols}
///     - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
///     - SQLColAttributeW(SQL_DESC_UNSIGNED)
///     - SQLColAttributeW(SQL_COLUMN_NAME)
///     - SQLColAttributeW(SQL_COLUMN_NULLABLE)
///     - SQLColAttributeW(SQL_DESC_TYPE_NAME)
///     - SQLColAttributeW(SQL_COLUMN_LENGTH)
///     - SQLColAttributeW(SQL_COLUMN_SCALE)
pub fn get_column_attributes(
    stmt: Handle,
    expected_col_count: SmallInt,
    mongosqltranslate_test_exp_vals: Option<Vec<Vec<Value>>>,
) {
    let str_len_ptr = &mut 0;
    let character_attrib_ptr: *mut std::ffi::c_void =
        Box::into_raw(Box::new([0; BUFFER_LENGTH as usize])) as *mut _;
    unsafe {
        let column_count_ptr = &mut 0;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLNumResultCols(stmt as HStmt, column_count_ptr),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
        );
        assert_eq!(expected_col_count, *column_count_ptr);

        let numeric_attribute_ptr = &mut 0;
        const FIELD_IDS: [Desc; 7] = [
            Desc::SQL_DESC_CONCISE_TYPE,
            Desc::SQL_DESC_UNSIGNED,
            Desc::SQL_DESC_NAME,
            Desc::SQL_DESC_NULLABLE,
            Desc::SQL_DESC_TYPE_NAME,
            Desc::SQL_DESC_LENGTH,
            Desc::SQL_DESC_SCALE,
        ];
        for col_num in 0..*column_count_ptr {
            FIELD_IDS.iter().enumerate().for_each(|(i, field_type)| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLColAttributeW(
                        stmt as HStmt,
                        (col_num + 1) as u16,
                        *field_type as u16,
                        character_attrib_ptr as Pointer,
                        BUFFER_LENGTH,
                        str_len_ptr,
                        numeric_attribute_ptr,
                    ),
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
                );

                if let Some(ref values) = mongosqltranslate_test_exp_vals {
                    let actual_val = match values[col_num as usize][i] {
                        Value::String(_) => json!(cstr::from_widechar_ref_lossy(
                            &*(character_attrib_ptr as *const [WideChar; BUFFER_LENGTH as usize])
                        )[0..(*str_len_ptr as usize / std::mem::size_of::<WideChar>())]
                            .to_string()),
                        Value::Number(_) => json!(*numeric_attribute_ptr),
                        _ => unreachable!("An unexpected metadata type was encountered."),
                    };

                    assert_eq!(actual_val, values[col_num as usize][i]);
                }
            });
        }

        let _ = Box::from_raw(character_attrib_ptr as *mut [u8; BUFFER_LENGTH as usize]);
    }
}

#[allow(dead_code)]
/// Helper function to bind columns.
pub unsafe fn bind_cols(
    stmt_handle: HStmt,
    target_types: Vec<(CDataType, Pointer, Len, *mut Len)>,
) {
    for (i, (target_type, binding_buffer, buffer_length, indicator)) in
        target_types.iter().enumerate()
    {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLBindCol(
                stmt_handle,
                (i + 1) as USmallInt,
                *target_type as SmallInt,
                *binding_buffer,
                *buffer_length,
                *indicator,
            ),
            "{}",
            get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
        )
    }
}

#[allow(dead_code)]
/// Helper function to execute the default query
///    SELECT * FROM integration_test.foo
/// via SQLExecDirectW.
pub unsafe fn exec_direct_default_query(stmt_handle: HStmt) {
    let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT * FROM integration_test.foo");
    query.push(0);
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLExecDirectW(stmt_handle, query.as_ptr(), SQL_NTS),
        "{}",
        get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
    );
}
