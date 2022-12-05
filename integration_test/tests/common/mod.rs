use odbc::ffi::SQL_NTS;
use odbc_sys::{
    AttrConnectionPooling, AttrOdbcVersion, ConnectionAttribute, DriverConnectOption,
    EnvironmentAttribute, HDbc, HEnv, Handle, HandleType, Pointer, SQLAllocHandle,
    SQLDriverConnectW, SQLSetConnectAttrW, SQLSetEnvAttr, SmallInt, SqlReturn,
};
use std::ptr::null_mut;
use std::{env, slice};

/// Generate the default connection setting defined for the tests using a connection string
/// of the form 'Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}'.
/// The default driver is 'ADF_ODBC_DRIVER' if not specified.
/// The default auth db is 'admin' if not specified.
pub fn generate_default_connection_str() -> String {
    let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
    let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
    let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");

    let auth_db = match env::var("ADF_TEST_LOCAL_AUTH_DB") {
        Ok(val) => val,
        Err(_e) => "admin".to_string(), //Default auth db
    };

    let db = env::var("ADF_TEST_LOCAL_DB");
    let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
        Ok(val) => val,
        Err(_e) => "ADF_ODBC_DRIVER".to_string(), //Default driver name
    };

    let mut connection_string = format!(
        "Driver={{{}}};USER={};PWD={};SERVER={};AUTH_SRC={};",
        driver, user_name, password, host, auth_db,
    );

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val + ";")),
        Err(_e) => (), // Do nothing
    };

    connection_string
}

/// Setup flow.
/// This will allocate a new environment handle and set ODBC_VERSION and CONNECTION_POOLING environment attributes.
/// Setup flow is:
///     - SQLAllocHandle(SQL_HANDLE_ENV)
///     - SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
///     - SQLSetEnvAttr(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
pub fn setup() -> odbc_sys::HEnv {
    let mut env: Handle = null_mut();

    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(HandleType::Env, null_mut(), &mut env as *mut Handle)
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::OdbcVersion,
                AttrOdbcVersion::Odbc3.into(),
                0,
            )
        );

        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetEnvAttr(
                env as HEnv,
                EnvironmentAttribute::ConnectionPooling,
                AttrConnectionPooling::OnePerHenv.into(),
                0,
            )
        );
    }

    env as HEnv
}

/// Generate the default connection string and returns :
/// - The connection handle
/// - The string used as the input connection string
/// - The retrieved output connection string
/// - The retrieved length of the output connection string
pub fn power_bi_connect(env_handle: HEnv) -> (odbc_sys::HDbc, String, String, SmallInt) {
    // Allocate a DBC handle
    let mut dbc: Handle = null_mut();
    let mut output_len = 0;
    let mut in_connection_string = "".to_string();
    let mut out_connection_string = "".to_string();
    unsafe {
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                &mut dbc as *mut Handle
            )
        );

        // Set the login timeout
        let login_timeout = 15;
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLSetConnectAttrW(
                dbc as HDbc,
                ConnectionAttribute::LoginTimeout,
                login_timeout as Pointer,
                0,
            )
        );

        // Generate the connection string and add a null terminator because PowerBi uses SQL_NTS for the length
        in_connection_string = generate_default_connection_str();
        let mut in_connection_string_encoded: Vec<u16> =
            in_connection_string.encode_utf16().collect();
        in_connection_string_encoded.push(0);

        let str_len_ptr = &mut 0;
        const BUFFER_LENGTH: SmallInt = 300;
        let out_connection_string_buff = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

        assert_ne!(
            SqlReturn::ERROR,
            SQLDriverConnectW(
                dbc as HDbc,
                null_mut(),
                in_connection_string_encoded.as_ptr(),
                SQL_NTS,
                out_connection_string_buff,
                BUFFER_LENGTH,
                str_len_ptr,
                DriverConnectOption::NoPrompt,
            )
        );

        output_len = *str_len_ptr;
        out_connection_string = String::from_utf16_lossy(slice::from_raw_parts(
            out_connection_string_buff,
            output_len as usize,
        ));
    }
    (
        dbc as HDbc,
        in_connection_string,
        out_connection_string,
        output_len,
    )
}
