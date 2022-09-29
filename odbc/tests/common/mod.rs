use lazy_static::lazy_static;
use mongoodbc::SQLGetDiagRecW;
use odbc_api::{Connection, Environment, Error};
use odbc_sys::{Handle, HandleType};
use std::env;

// Allocate a new environment handle.
// Most tests will only need one and this should be part of the setup mechanism.
lazy_static! {
    pub static ref ODBC_ENV: Environment = Environment::new().unwrap();
}

/// Generate the default connection setting defined for the tests using a connection string
/// of the form 'Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}'.
/// The default driver is 'ADF_ODBC_DRIVER' if not specified.
/// The default auth db is 'admin' if not specified.
fn generate_default_connection_str() -> String {
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
        "Driver={};USER={};PWD={};SERVER={};AUTH_SRC={};",
        driver, user_name, password, host, auth_db,
    );

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val)),
        Err(_e) => (), // Do nothing
    };

    connection_string
}

/// Connect using the given connection string or the default settings if no connection string are provided.
pub fn connect(connection_string: Option<&str>) -> Result<Connection<'_>, Error> {
    match connection_string {
        Some(str) => ODBC_ENV.connect_with_connection_string(str),
        None => ODBC_ENV.connect_with_connection_string(generate_default_connection_str().as_str()),
    }
}

// Verifies that the expected SQL State, message text, and native error in the handle match
// the expected input
pub fn verify_sql_diagnostics(
    handle_type: HandleType,
    handle: Handle,
    record_number: i16,
    expected_sql_state: &str,
    expected_message_text: &str,
    mut expected_native_err: i32,
) {
    let text_length_ptr = &mut 0;
    let actual_sql_state = &mut [0u16; 6] as *mut _;
    let actual_message_text = &mut [0u16; 512] as *mut _;
    let actual_native_error = &mut 0;
    let _ = SQLGetDiagRecW(
        handle_type,
        handle as *mut _,
        record_number,
        actual_sql_state,
        actual_native_error,
        actual_message_text,
        1024,
        text_length_ptr,
    );
    let mut expected_sql_state_encoded: Vec<u16> = expected_sql_state.encode_utf16().collect();
    expected_sql_state_encoded.push(0);
    let actual_message_length = *text_length_ptr as usize;
    unsafe {
        assert_eq!(
            expected_message_text,
            &(String::from_utf16_lossy(&*(actual_message_text as *const [u16; 256])))
                [0..actual_message_length],
        );
        assert_eq!(
            String::from_utf16(&*(expected_sql_state_encoded.as_ptr() as *const [u16; 6])).unwrap(),
            String::from_utf16(&*(actual_sql_state as *const [u16; 6])).unwrap()
        );
    }
    assert_eq!(&mut expected_native_err as &mut i32, actual_native_error);
}
