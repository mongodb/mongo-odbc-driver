use crate::{odbc_uri::ODBCUri, MongoConnection, TypeMode};
use cstr::{input_text_to_string_w, write_string_to_buffer, WideChar};

/// atlas_sql_test_connection returns true if a connection can be established
/// with the provided connection string.
/// If the connection fails, the error message is written to the buffer.
///
/// # Arguments
/// * `connection_string` - A null-terminated widechar string containing the connection string.
/// * `buffer` - A buffer to write the error message to, in widechar chars.
/// * `buffer_in_len` - The length of the buffer, in widechar chars.
/// * `buffer_out_length` - The length of data written to buffer, in widechar chars.
///
/// # Safety
/// Because this function is called from C, it is unsafe.
///
pub fn foo() {
    println!("foo");
}
// #[no_mangle]
// pub unsafe extern "C" fn atlas_sql_test_connection(
//     connection_string: *const WideChar,
//     buffer: *const WideChar,
//     buffer_in_len: usize,
//     buffer_out_len: *mut u16,
// ) -> bool {
//     let marker = -1i8;
//     let conn_str = unsafe { input_text_to_string_w(connection_string, marker as usize) };
//     if let Ok(mut odbc_uri) = ODBCUri::new(conn_str) {
//         match odbc_uri.try_into_client_options() {
//             Ok(client_options) => {
//                 match MongoConnection::connect(
//                     client_options,
//                     odbc_uri.get("database").map(|s| s.to_owned()),
//                     None,
//                     Some(30),
//                     TypeMode::Standard,
//                 ) {
//                     Ok(_) => true,
//                     Err(e) => {
//                         let len = write_string_to_buffer(
//                             &e.to_string(),
//                             buffer_in_len,
//                             buffer as *mut WideChar,
//                         );
//                         *buffer_out_len = len;
//                         false
//                     }
//                 }
//             }
//             Err(e) => {
//                 let len =
//                     write_string_to_buffer(&e.to_string(), buffer_in_len, buffer as *mut WideChar);
//                 *buffer_out_len = len;
//                 false
//             }
//         }
//     } else {
//         let len = write_string_to_buffer(
//             "Invalid connection string.",
//             buffer_in_len,
//             buffer as *mut WideChar,
//         );
//         *buffer_out_len = len;
//         false
//     }
// }

// // Tests require a local adf to be running.
// #[cfg(test)]
// mod test {
//     use super::atlas_sql_test_connection;
//     use constants::DRIVER_NAME;
//     use cstr::{input_text_to_string_w, to_widechar_ptr};
//     use std::env;

//     #[test]
//     fn successful_connection() {
//         let mut buffer = [0; 1024];
//         let mut buffer_len = 0;
//         let result = unsafe {
//             atlas_sql_test_connection(
//                 to_widechar_ptr(&generate_connection_str(None, None)).0 as *const cstr::WideChar,
//                 buffer.as_mut_ptr(),
//                 buffer.len(),
//                 &mut buffer_len,
//             )
//         };
//         assert!(result);
//     }

//     #[test]
//     fn bad_credentials() {
//         let mut buffer = [0; 1024];
//         let mut buffer_len = 0;
//         let result = unsafe {
//             atlas_sql_test_connection(
//                 to_widechar_ptr(&generate_connection_str(None, Some("hunter2".into()))).0
//                     as *const cstr::WideChar,
//                 buffer.as_mut_ptr(),
//                 buffer.len(),
//                 &mut buffer_len,
//             )
//         };
//         assert!(!result);
//         assert!(unsafe {
//             input_text_to_string_w(buffer.as_ptr(), buffer_len as usize)
//                 .to_lowercase()
//                 .contains("authentication failed")
//         });
//     }

//     #[test]
//     #[cfg(feature = "bad_host")]
//     fn bad_host() {
//         let mut buffer = [0; 1024];
//         let mut buffer_len = 0;
//         let result = unsafe {
//             atlas_sql_test_connection(
//                 to_widechar_ptr(&generate_connection_str(
//                     Some("example.net:30000".into()),
//                     None,
//                 ))
//                 .0 as *const cstr::WideChar,
//                 buffer.as_ptr(),
//                 buffer.len(),
//                 &mut buffer_len,
//             )
//         };
//         assert!(!result);
//         assert!(unsafe {
//             input_text_to_string_w(buffer.as_mut_ptr(), buffer_len as usize)
//                 .to_lowercase()
//                 .contains("server selection timeout")
//         });
//     }

//     // lifted and modified from integration_test\tests\connection_tests.rs
//     // this cannot be included due to dependendy issues
//     fn generate_connection_str(host: Option<String>, password: Option<String>) -> String {
//         let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
//         let pwd = password
//             .unwrap_or(env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set"));
//         let server = host
//             .unwrap_or(env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set"));

//         let db = env::var("ADF_TEST_LOCAL_DB");
//         let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
//             Ok(val) => val,
//             Err(_e) => DRIVER_NAME.to_string(), //Default driver name
//         };

//         let mut connection_string =
//             format!("Driver={{{driver}}};USER={user_name};PWD={pwd};SERVER={server};");

//         // If a db is specified add it to the connection string
//         match db {
//             Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val + ";")),
//             Err(_e) => (), // Do nothing
//         };

//         connection_string
//     }
// }
