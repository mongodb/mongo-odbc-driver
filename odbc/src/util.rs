use crate::handles::SQLState;
use odbc_sys::{SmallInt, SqlReturn, WChar};
use std::{cmp::min, ptr::copy_nonoverlapping};

/// set_sql_state writes the given sql state to the [`output_ptr`].
pub fn set_sql_state(sql_state: SQLState, output_ptr: *mut WChar) {
    let sql_state = &format!("{}\0", sql_state);
    let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
    unsafe {
        copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
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
        let mut message_u16 = error_message.encode_utf16().collect::<Vec<u16>>();
        let message_len = message_u16.len();
        let num_chars = min(message_len + 1, buffer_len);
        message_u16.resize(num_chars - 1, 0);
        message_u16.push('\u{0}' as u16);
        copy_nonoverlapping(message_u16.as_ptr(), output_ptr, num_chars);
        // Store the number of characters in the error message string, excluding the
        // null terminator, in text_length_ptr
        *text_length_ptr = (num_chars - 1) as SmallInt;
        if num_chars < message_len {
            SqlReturn::SUCCESS_WITH_INFO
        } else {
            SqlReturn::SUCCESS
        }
    }
}
