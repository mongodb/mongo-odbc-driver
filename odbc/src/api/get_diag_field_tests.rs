use crate::{api::errors::ODBCError, handles::definitions::*, SQLGetDiagFieldW};
use odbc_sys::{HandleType, SqlReturn};

const UNIMPLEMENTED_FUNC: &str = "HYC00\0";

mod unit {
    use super::*;
    use std::ffi::c_void;

    fn validate_integer_diag_field(
        handle_type: HandleType,
        handle: *mut MongoHandle,
        diag_identifier: i16,
        expected_value: i64,
    ) {
        let diag_info_ptr = &mut 0i64 as *mut _ as *mut c_void;
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    diag_identifier,
                    diag_info_ptr,
                    0,
                    &mut 0
                )
            );
            assert_eq!(expected_value, *(diag_info_ptr as *const i64));
        }
    }

    fn validate_message_text(handle_type: HandleType, handle: *mut MongoHandle) {
        use cstr::WideChar;
        use std::mem::size_of;
        const ERROR_MESSAGE: &str = "[MongoDB][API] The feature SQLDrivers is not implemented\0";
        let message_text = &mut [0; 57 * size_of::<WideChar>()] as *mut _ as *mut c_void;
        let string_length_ptr = &mut 0;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    message_text,
                    57 * size_of::<WideChar>() as i16,
                    string_length_ptr
                )
            );
            assert_eq!(
                ERROR_MESSAGE,
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 57]))
            );
            assert_eq!(56 * size_of::<WideChar>() as i16, *string_length_ptr);
        }
    }

    fn validate_sql_state(handle_type: HandleType, handle: *mut MongoHandle) {
        use cstr::WideChar;
        use std::mem::size_of;
        let message_text = &mut [0; 6 * size_of::<WideChar>()] as *mut _ as *mut c_void;
        let string_length_ptr = &mut 0;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    4, //DiagType::SQL_DIAG_SQLSTATE
                    message_text,
                    6 * size_of::<WideChar>() as i16,
                    string_length_ptr
                )
            );
            assert_eq!(
                UNIMPLEMENTED_FUNC,
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 6]))
            );
            assert_eq!(5 * size_of::<WideChar>() as i16, *string_length_ptr);
        }
    }

    fn validate_return_code(handle_type: HandleType, handle: *mut MongoHandle) {
        /*
           The return code is always implemented by the driver manager, per the spec.
           Thus, calling SQLGetDiagField with type SQL_DIAG_RETURNCODE is essentially
           a no-op. Verify this by checking we get sqlsucces, and buffers remain unchanged.
        */
        let message_text = &mut [116, 101, 115, 116, 0] as *mut _;
        let string_length_ptr = &mut 0;
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    1, // DiagType::SQL_DIAG_RETURNCODE
                    message_text as *mut _ as *mut c_void,
                    10,
                    string_length_ptr
                )
            );
            // checking input pointer was not altered in any way, and we just pass through SUCCESS
            assert_eq!([116, 101, 115, 116, 0], *message_text);
            assert_eq!(0, *string_length_ptr);
        }
    }

    #[test]
    fn test_simple() {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
        ));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            conn_handle,
            StatementState::Allocated,
        ));
        let handles: Vec<(HandleType, *mut MongoHandle)> = vec![
            (HandleType::Env, env_handle),
            (HandleType::Dbc, conn_handle),
            (HandleType::Stmt, stmt_handle),
        ];
        handles.iter().for_each(|(handle_type, handle)| {
            unsafe {
                (**handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
            }
            validate_message_text(*handle_type, *handle);
            validate_sql_state(*handle_type, *handle);
            // SQL_DIAG_NATIVE
            validate_integer_diag_field(*handle_type, *handle, 5, 0);
            // SQL_DIAG_NUMBER
            validate_integer_diag_field(*handle_type, *handle, 2, 1);
            validate_return_code(*handle_type, *handle);

            //statement only
            if *handle_type == HandleType::Stmt {
                // SQL_DIAG_ROW_NUMBER
                validate_integer_diag_field(*handle_type, *handle, -1248, -2i64);
                // SQL_DIAG_ROW_COUNT
                validate_integer_diag_field(*handle_type, *handle, 3, 0i64);
            }
        });
    }

    #[test]
    fn test_error_message() {
        use cstr::WideChar;
        use std::mem::size_of;

        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        // Initialize buffers
        let message_text = &mut [0; 500 * size_of::<WideChar>()] as *mut _ as *mut c_void;
        let string_length_ptr = &mut 0;

        unsafe {
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
            // Buffer is too small to hold the entire error message and the null terminator
            // (0 < length < 57)
            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    message_text,
                    15 * size_of::<WideChar>() as i16,
                    string_length_ptr
                )
            );
            assert_eq!(
                "[MongoDB][API]\0",
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 15]))
            );
            // Error message string where some characters are composed of more than one byte.
            // 1 < RecNumber =< number of diagnostic records.
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDriv✐𑜲"));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    2,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    message_text,
                    57 * size_of::<WideChar>() as i16,
                    string_length_ptr
                )
            );
            assert_eq!(
                "[MongoDB][API] The feature SQLDriv✐𑜲 is not implemented\0",
                if std::mem::size_of::<WideChar>() == std::mem::size_of::<u16>() {
                    cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 57]))
                } else {
                    cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 56]))
                }
            );
        }
    }

    #[test]
    fn test_invalid_ops() {
        use cstr::WideChar;
        use std::mem::size_of;
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        unsafe {
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
            // Buffer length < 0
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    (&mut [0; 6 * size_of::<WideChar>()]) as *mut _ as *mut c_void,
                    -1,
                    &mut 0
                )
            );
            // Record number <= 0
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    0,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    (&mut [0; 6 * size_of::<WideChar>()]) as *mut _ as *mut c_void,
                    57,
                    &mut 0
                )
            );
            // Record number > number of diagnostic records
            assert_eq!(
                SqlReturn::NO_DATA,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    3,
                    6, //DiagType::SQL_DIAG_MESSAGE_TEXT
                    (&mut [0; 6 * size_of::<WideChar>()]) as *mut _ as *mut c_void,
                    57,
                    &mut 0
                )
            );
            // Header fields that require a statement handle
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    -1249, // DiagType::SQL_DIAG_ROW_COUNT
                    (&mut 0) as *mut _ as *mut c_void,
                    10,
                    &mut 0
                )
            );
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    -1248, // DiagType::SQL_DIAG_ROW_NUMBER
                    (&mut 0) as *mut _ as *mut c_void,
                    10,
                    &mut 0
                )
            );
            // make a call for an unimplemented diagnostic type
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    12, // DiagType::SQL_DIAG_DYNAMIC_FUNCTION_CODE
                    &mut 0 as *mut _ as *mut c_void,
                    1,
                    &mut 0
                )
            );
        }
    }
}
