use crate::handles::{MongoHandle, ODBCError};
use odbc_sys::{Handle, HandleType, SmallInt, SqlReturn, WChar};
use std::{cmp::min, ptr::copy};

/// add_diag_info creates a new `ODBCError` object and appends it to the
/// given handle's `errors` field.
pub fn add_diag_info(
    handle_type: HandleType,
    handle: Handle,
    sql_state: String,
    error_message: String,
    native_err_code: i32,
    component: String,
) -> Result<(), ()> {
    let error = ODBCError {
        sql_state,
        error_message,
        native_err_code,
        component,
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
    let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>().as_ptr();
    unsafe {
        copy(state_u16, output_ptr, 6);
    }
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
        let msg_u16 = error_message.encode_utf16().collect::<Vec<u16>>();
        let num_chars = min(msg_u16.len() + 1, buffer_len);
        let mut msg = msg_u16[..num_chars - 1].to_vec();
        msg.push('\u{0}' as u16);
        copy(msg.as_ptr(), output_ptr, num_chars);
        *text_length_ptr = num_chars as SmallInt;
        if num_chars < msg_u16.len() {
            SqlReturn::SUCCESS_WITH_INFO
        } else {
            SqlReturn::SUCCESS
        }
    }
}
