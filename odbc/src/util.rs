use crate::handles::{MongoHandle, ODBCError};
use odbc_sys::{Handle, HandleType, SmallInt, SqlReturn, WChar};
use std::{cmp::min, ptr::copy};

/// set_handle_state writes the error code [`sql_state`] to the field `sql_state`
/// in [`handle`].
pub fn set_handle_state(
    handle_type: HandleType,
    handle: Handle,
    sql_state: String,
    error_message: String,
    native_err_code: i32,
) -> Result<(), ()> {
    let error = ODBCError {
        sql_state,
        error_message,
        native_err_code,
    };
    match handle_type {
        HandleType::Env => {
            let env = unsafe { (*(handle as *mut MongoHandle)).as_env().ok_or(())? };
            let mut env_contents = env.write().unwrap();
            env_contents.errors.push(error);
        }
        HandleType::Dbc => {
            let dbc = unsafe { (*(handle as *mut MongoHandle)).as_connection().ok_or(())? };
            let mut dbc_contents = dbc.write().unwrap();
            dbc_contents.errors.push(error);
        }
        HandleType::Stmt => {
            let stmt = unsafe { (*(handle as *mut MongoHandle)).as_statement().ok_or(())? };
            let mut stmt_contents = stmt.write().unwrap();
            stmt_contents.errors.push(error);
        }
        HandleType::Desc => {
            let desc = unsafe { (*(handle as *mut MongoHandle)).as_descriptor().ok_or(())? };
            let mut desc_contents = desc.write().unwrap();
            desc_contents.errors.push(error);
        }
    };
    Ok(())
}

/// set_sql_state writes the given sql state to the [`output_ptr`].
pub fn set_sql_state(mut sql_state: String, output_ptr: *mut WChar) {
    sql_state.push('\0');
    cp_str_to_buffer(sql_state, output_ptr, 6);
}

/// set_error_message writes [`error_message`] to the [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the error message should be truncated
/// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
/// should be stored in [`text_length_ptr`].
pub fn set_error_message(
    error_message: String,
    output_ptr: *mut WChar,
    buffer_len: usize,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsafe {
        // Check if the entire error message plus a null terminator can fit in the buffer;
        // we should truncate the error message if it's too long.
        let num_chars = min(error_message.len() + 1, buffer_len);
        let msg = &mut error_message[..num_chars - 1].to_string();
        msg.push('\0');
        *text_length_ptr = num_chars as SmallInt;
        cp_str_to_buffer(msg.clone(), output_ptr, buffer_len);
        if num_chars < error_message.len() {
            SqlReturn::SUCCESS_WITH_INFO
        } else {
            SqlReturn::SUCCESS
        }
    }
}

/// cp_str_to_buffer copies [`s`] into [`buffer`].
fn cp_str_to_buffer(s: String, buffer: *mut WChar, buffer_len: usize) {
    let str_u16 = s.encode_utf16().collect::<Vec<u16>>().as_ptr();
    unsafe {
        copy(str_u16, buffer, buffer_len);
    }
}
