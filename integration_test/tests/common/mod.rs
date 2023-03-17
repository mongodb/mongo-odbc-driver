use constants::DRIVER_NAME;
use cstr::WideChar;
use odbc_sys::{Handle, HandleType, SQLGetDiagRecW, SqlReturn};
use std::{env, slice};

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
