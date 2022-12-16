use crate::{
    api::{definitions::DiagType, errors::ODBCError},
    handles::definitions::*,
    SQLGetDiagFieldW,
};
use odbc_sys::{HandleType, SqlReturn};

const UNIMPLEMENTED_FUNC: &str = "HYC00\0";

mod unit {
    use super::*;
    use std::ffi::c_void;

    fn validate_integer_diag_field(
        handle_type: HandleType,
        handle: *mut MongoHandle,
        diag_identifier: DiagType,
        expected_value: i32,
    ) {
        let diag_info_ptr = &mut 0 as *mut _ as *mut c_void;
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
            assert_eq!(expected_value, *(diag_info_ptr as *const i32));
        }
    }

    fn validate_message_text(handle_type: HandleType, handle: *mut MongoHandle) {
        const ERROR_MESSAGE: &str = "[MongoDB][API] The feature SQLDrivers is not implemented\0";
        let message_text = &mut [0u16; 57] as *mut _ as *mut c_void;
        let string_length_ptr = &mut 0;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    DiagType::MessageText,
                    message_text,
                    57,
                    string_length_ptr
                )
            );
            assert_eq!(
                ERROR_MESSAGE,
                String::from_utf16(&*(message_text as *const [u16; 57])).unwrap()
            );
            assert_eq!(56, *string_length_ptr);
        }
    }

    fn validate_sql_state(handle_type: HandleType, handle: *mut MongoHandle) {
        let message_text = &mut [0u16; 6] as *mut _ as *mut c_void;
        let string_length_ptr = &mut 0;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    DiagType::SqlState,
                    message_text,
                    6,
                    string_length_ptr
                )
            );
            assert_eq!(
                UNIMPLEMENTED_FUNC,
                String::from_utf16(&*(message_text as *const [u16; 6])).unwrap()
            );
            assert_eq!(5, *string_length_ptr);
        }
    }

    fn validate_return_code(handle_type: HandleType, handle: *mut MongoHandle) {
        let message_text = &mut [116u16, 101, 115, 116, 0];
        let string_length_ptr = &mut 0;
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    handle_type,
                    handle as *mut _,
                    1,
                    DiagType::ReturnCode,
                    message_text as *mut _ as *mut c_void,
                    10,
                    string_length_ptr
                )
            );
            // checking input pointer was not altered in any way, and we just pass through SUCESS
            assert_eq!(
                "test\0",
                String::from_utf16(&*(message_text as *const [u16; 5])).unwrap()
            );
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
            std::ptr::null_mut(),
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
            validate_integer_diag_field(*handle_type, *handle, DiagType::Native, 0);
            validate_integer_diag_field(*handle_type, *handle, DiagType::Number, 1);
            validate_return_code(*handle_type, *handle);

            //statement only
            if *handle_type == HandleType::Stmt {
                validate_integer_diag_field(*handle_type, *handle, DiagType::RowNumber, 0);
                validate_integer_diag_field(*handle_type, *handle, DiagType::RowCount, 0);
            }
        });
    }

    #[test]
    fn test_error_message() {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        // Initialize buffers
        let message_text = &mut [0u16; 57] as *mut _ as *mut c_void;
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
                    DiagType::MessageText,
                    message_text,
                    15,
                    string_length_ptr
                )
            );
            assert_eq!(
                "[MongoDB][API]\0",
                String::from_utf16(&*(message_text as *const [u16; 15])).unwrap()
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
                    DiagType::MessageText,
                    message_text,
                    57,
                    string_length_ptr
                )
            );
            assert_eq!(
                "[MongoDB][API] The feature SQLDriv✐𑜲 is not implemented\0",
                String::from_utf16(&*(message_text as *const [u16; 57])).unwrap()
            );
        }
    }

    #[test]
    fn test_invalid_ops() {
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
                    DiagType::MessageText,
                    (&mut [0u16; 6]) as *mut _ as *mut c_void,
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
                    DiagType::MessageText,
                    (&mut [0u16; 6]) as *mut _ as *mut c_void,
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
                    DiagType::MessageText,
                    (&mut [0u16; 6]) as *mut _ as *mut c_void,
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
                    DiagType::RowCount,
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
                    DiagType::RowNumber,
                    (&mut 0) as *mut _ as *mut c_void,
                    10,
                    &mut 0
                )
            );
        }
    }

    /*
       TODO: This test should be removed when we implement the rest of the diag field types
    */
    #[test]
    fn test_invalid_diag_identifier() {
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        unsafe {
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));

            // check first we have exactly one error in the handle
            let num_errors_buffer = &mut 0u64 as *mut _ as *mut c_void;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    0,
                    DiagType::Number,
                    num_errors_buffer,
                    0,
                    &mut 0
                )
            );
            assert_eq!(1, *(num_errors_buffer as *const u64));

            // make a call for an unimplemented diagnostic type
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    DiagType::DynamicFunctionCode,
                    &mut 0 as *mut _ as *mut c_void,
                    1,
                    &mut 0
                )
            );
            // check the correct diagnostic was added for this error
            let num_errors_buffer = &mut 0 as *mut _ as *mut c_void;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    0,
                    DiagType::Number,
                    num_errors_buffer,
                    0,
                    &mut 0
                )
            );
            assert_eq!(2, *(num_errors_buffer as *const i32));

            // validating error text
            let message_text = &mut [0u16; 78] as *mut _ as *mut c_void;
            let string_length_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    HandleType::Env,
                    env_handle as *mut _,
                    2,
                    DiagType::MessageText,
                    message_text,
                    78,
                    string_length_ptr
                )
            );
            assert_eq!(
                "[MongoDB][API] The diag identifier value DynamicFunctionCode is not supported\0",
                String::from_utf16(&*(message_text as *const [u16; 78])).unwrap()
            );
            assert_eq!(77, *string_length_ptr);
        }
    }
}
