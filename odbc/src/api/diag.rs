use crate::{
    api::{
        data::i16_len,
        definitions::{DiagType, SQL_ROW_NUMBER_UNKNOWN},
    },
    errors::ODBCError,
};
use odbc_sys::Pointer;
use odbc_sys::{Char, Integer, SmallInt, SqlReturn, WChar};
use std::ptr::copy_nonoverlapping;

///
/// set_sql_state writes the given sql state to the [`output_ptr`].
///
/// # Safety
/// This writes to a raw C-pointer
///
pub unsafe fn set_sql_state(sql_state: &str, output_ptr: *mut Char) {
    if output_ptr.is_null() {
        return;
    }
    let sql_state = &format!("{sql_state}\0");
    let state_u8 = sql_state.bytes().collect::<Vec<u8>>();
    copy_nonoverlapping(state_u8.as_ptr(), output_ptr, 6);
}

///
/// set_sql_statew writes the given sql state to the [`output_ptr`].
///
/// # Safety
/// This writes to a raw C-pointer
///
pub unsafe fn set_sql_statew(sql_state: &str, output_ptr: *mut WChar) {
    if output_ptr.is_null() {
        return;
    }
    let sql_state = &format!("{sql_state}\0");
    let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
    copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
}

///
/// get_diag_rec copies the given ODBC error's diagnostic information
/// into the provided pointers.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn get_diag_rec(
    error: &ODBCError,
    state: *mut Char,
    message_text: *mut Char,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
    native_error_ptr: *mut Integer,
) -> SqlReturn {
    if !native_error_ptr.is_null() {
        *native_error_ptr = error.get_native_err_code();
    }
    set_sql_state(error.get_sql_state(), state);
    let message = format!("{error}");
    i16_len::set_output_string(
        &message,
        message_text,
        buffer_length as usize,
        text_length_ptr,
    )
}

///
/// get_diag_recw copies the given ODBC error's diagnostic information
/// into the provided pointers.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn get_diag_recw(
    error: &ODBCError,
    state: *mut WChar,
    message_text: *mut WChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
    native_error_ptr: *mut Integer,
) -> SqlReturn {
    if !native_error_ptr.is_null() {
        *native_error_ptr = error.get_native_err_code();
    }
    set_sql_statew(error.get_sql_state(), state);
    let message = format!("{error}");
    i16_len::set_output_wstring(
        &message,
        message_text,
        buffer_length as usize,
        text_length_ptr,
    )
}

///
/// get_stmt_diag_field copies a part of the given ODBC error's diagnostic information
/// into the provided pointer.
///
/// # Safety
/// This writes to a raw C-pointer
///
pub unsafe fn get_stmt_diag_field(diag_identifier: DiagType, diag_info_ptr: Pointer) -> SqlReturn {
    // NOTE: at the moment, this could be merged with get_diag_field. However, as part of SQL-1152,
    // some functionality will be specific to the statement handle, and thus warrants a separate function
    match diag_identifier {
        // default to 0, mirroring the behavior in SQLRowCount
        DiagType::SQL_DIAG_ROW_COUNT => {
            i16_len::set_output_fixed_data(&0isize, diag_info_ptr, &mut 0)
        }
        DiagType::SQL_DIAG_ROW_NUMBER => {
            // default to unknown, as at the moment statement handels don't update their row number attribute
            i16_len::set_output_fixed_data(&SQL_ROW_NUMBER_UNKNOWN, diag_info_ptr, &mut 0)
        }
        // this should not be reachable if match branches here match those in SQLGetDiagFieldW
        _ => SqlReturn::ERROR,
    }
}

///
/// get_diag_field copies a part of the given ODBC error's diagnostic information
/// into the provided pointers.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn get_diag_field(
    errors: &Vec<ODBCError>,
    diag_identifier: DiagType,
    diag_info_ptr: Pointer,
    record_number: i16,
    buffer_length: i16,
    string_length_ptr: *mut i16,
    is_wstring: bool,
) -> SqlReturn {
    // NOTE: number is dependent on the list of errors, but is a header, hence separating it from the match
    if diag_identifier == DiagType::SQL_DIAG_NUMBER {
        *(diag_info_ptr as *mut usize) = errors.len();
        SqlReturn::SUCCESS
    } else {
        if buffer_length < 0 || record_number < 1 {
            return SqlReturn::ERROR;
        }
        let rec_number = (record_number - 1) as usize;
        match errors.get(rec_number) {
            Some(error) => {
                match diag_identifier {
                    // NOTE: return code is handled by driver manager; just return success
                    DiagType::SQL_DIAG_RETURNCODE => SqlReturn::SUCCESS,
                    DiagType::SQL_DIAG_SQLSTATE => match is_wstring {
                        true => i16_len::set_output_wstring(
                            error.get_sql_state(),
                            diag_info_ptr as *mut u16,
                            buffer_length as usize,
                            string_length_ptr,
                        ),
                        false => i16_len::set_output_string(
                            error.get_sql_state(),
                            diag_info_ptr as *mut u8,
                            buffer_length as usize,
                            string_length_ptr,
                        ),
                    },
                    DiagType::SQL_DIAG_NATIVE => i16_len::set_output_fixed_data(
                        &error.get_native_err_code(),
                        diag_info_ptr,
                        std::ptr::null_mut::<i16>(),
                    ),
                    DiagType::SQL_DIAG_MESSAGE_TEXT => {
                        let message = format!("{error}");
                        match is_wstring {
                            true => i16_len::set_output_wstring(
                                &message,
                                diag_info_ptr as *mut u16,
                                buffer_length as usize,
                                string_length_ptr,
                            ),
                            false => i16_len::set_output_string(
                                &message,
                                diag_info_ptr as *mut u8,
                                buffer_length as usize,
                                string_length_ptr,
                            ),
                        }
                    }
                    // this should not be reachable if match branches here mirror those in SQLGetDiagFieldW
                    _ => SqlReturn::ERROR,
                }
            }
            _ => SqlReturn::NO_DATA,
        }
    }
}
