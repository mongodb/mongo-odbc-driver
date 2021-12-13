use crate::handles::MongoHandle;
use odbc_sys::{Char, Handle, HandleType, SmallInt};
use std::{cmp::min, ptr::copy_nonoverlapping};

/// set_handle_state writes the error code [`sql_state`] to the field `sql_state`
/// in [`handle`].
pub fn set_handle_state(
    handle_type: HandleType,
    handle: Handle,
    sql_state: &str,
    error_message: &str,
) -> Result<(), ()> {
    match handle_type {
        HandleType::Env => {
            let env = unsafe { (*(handle as *mut MongoHandle)).as_env().ok_or(())? };
            let mut env_contents = env.write().unwrap();
            env_contents.sql_states.push(sql_state.to_string());
            env_contents.error_messages.push(error_message.to_string());
        }
        HandleType::Dbc => {
            let dbc = unsafe { (*(handle as *mut MongoHandle)).as_connection().ok_or(())? };
            let mut dbc_contents = dbc.write().unwrap();
            dbc_contents.sql_states.push(sql_state.to_string());
            dbc_contents.error_messages.push(error_message.to_string());
        }
        HandleType::Stmt => {
            let stmt = unsafe { (*(handle as *mut MongoHandle)).as_statement().ok_or(())? };
            let mut stmt_contents = stmt.write().unwrap();
            stmt_contents.sql_states.push(sql_state.to_string());
            stmt_contents.error_messages.push(error_message.to_string());
        }
        HandleType::Desc => {
            let desc = unsafe { (*(handle as *mut MongoHandle)).as_descriptor().ok_or(())? };
            let mut desc_contents = desc.write().unwrap();
            desc_contents.sql_states.push(sql_state.to_string());
            desc_contents.error_messages.push(error_message.to_string());
        }
    };
    Ok(())
}

/// set_sql_state writes the given sql state to the [`output_ptr`].
pub fn set_sql_state(mut sql_state: String, output_ptr: *mut Char) {
    unsafe {
        let state = std::mem::transmute::<*mut u8, *mut Char>(sql_state.as_mut_ptr());
        std::ptr::copy_nonoverlapping(state, output_ptr, 5)
    }
}

/// set_error_message writes [`error_message`] to the [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the error message should be truncated
/// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
/// should be stored in [`text_length_ptr`].
pub fn set_error_message(
    mut error_message: String,
    output_ptr: *mut Char,
    buffer_len: usize,
    text_length_ptr: *mut SmallInt,
) {
    unsafe {
        let msg = std::mem::transmute::<*mut u8, *mut Char>(error_message.as_mut_ptr());
        let num_chars = min(error_message.len(), buffer_len);
        *text_length_ptr = num_chars as SmallInt;
        copy_nonoverlapping(msg, output_ptr, num_chars);
    }
}
