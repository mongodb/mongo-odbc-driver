use cstr::{
    input_text_to_string_w, write_string_to_buffer, WideChar,
};
use definitions::{Integer, SQL_NTS_ISIZE};
use crate::{odbc_uri::ODBCUri, MongoConnection, TypeMode};

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

#[no_mangle]
pub unsafe extern "C" fn atlas_sql_test_connection(
    connection_string: *const WideChar,
    buffer: *const WideChar,
    buffer_in_len: usize,
    buffer_out_len: *mut Integer,
) -> bool {
    let conn_str = unsafe { input_text_to_string_w(connection_string, SQL_NTS_ISIZE) };
    if let Ok(mut odbc_uri) = ODBCUri::new(conn_str) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client_options =
            runtime.block_on(async { odbc_uri.try_into_client_options().await });
        match client_options {
            Ok(client_options) => {
                match MongoConnection::connect(
                    client_options,
                    odbc_uri.get("database").map(|s| s.to_owned()),
                    None,
                    Some(30),
                    TypeMode::Standard,
                    Some(runtime),
                    None,
                ) {
                    Ok(_) => true,
                    Err(e) => {
                        let len = write_string_to_buffer(
                            &e.to_string(),
                            buffer_in_len as isize,
                            buffer as *mut WideChar,
                        );
                        *buffer_out_len = len as Integer;
                        false
                    }
                }
            }
            Err(e) => {
                let len = write_string_to_buffer(
                    &e.to_string(),
                    buffer_in_len as isize,
                    buffer as *mut WideChar,
                );
                *buffer_out_len = len as Integer;
                false
            }
        }
    } else {
        let len = write_string_to_buffer(
            "Invalid connection string.",
            buffer_in_len as isize,
            buffer as *mut WideChar,
        );
        *buffer_out_len = len as Integer;
        false
    }
}
