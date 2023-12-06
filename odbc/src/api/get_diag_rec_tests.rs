use crate::{api::errors::ODBCError, handles::definitions::*, SQLGetDiagRecW};
use odbc_sys::{HandleType, SqlReturn};

const UNIMPLEMENTED_FUNC: &str = "HYC00\0";

mod unit {
    use super::*;
    #[test]
    fn test_simple() {
        use cstr::WideChar;
        fn validate_diag_rec(handle_type: HandleType, handle: *mut MongoHandle) {
            const ERROR_MESSAGE: &str =
                "[MongoDB][API] The feature SQLDrivers is not implemented\0";

            // Initialize buffers
            let mut sql_state: [WideChar; 6] = [0; 6];
            let sql_state = &mut sql_state as *mut WideChar;
            // Note: len(ERROR_MESSAGE) = 57
            let mut message_text: [WideChar; 57] = [0; 57];
            let message_text = &mut message_text as *mut WideChar;
            let text_length_ptr = &mut 0;
            let native_err_ptr = &mut 0;

            unsafe {
                (*handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetDiagRecW(
                        handle_type,
                        handle as *mut _,
                        1,
                        sql_state,
                        native_err_ptr,
                        message_text,
                        60, // Some number >= 57
                        text_length_ptr,
                    )
                );
                assert_eq!(
                    UNIMPLEMENTED_FUNC,
                    cstr::from_widechar_ref_lossy(&*(sql_state as *const [WideChar; 6]))
                );
                assert_eq!(
                    ERROR_MESSAGE,
                    cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 57]))
                );
                // text_length_ptr includes a byte for null termination.
                assert_eq!(56, *text_length_ptr);
                assert_eq!(0, *native_err_ptr);
            }
        }

        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        validate_diag_rec(HandleType::Env, env_handle);

        let conn_handle: *mut _ = &mut MongoHandle::Connection(Connection::with_state(
            env_handle,
            ConnectionState::Allocated,
        ));
        validate_diag_rec(HandleType::Dbc, conn_handle);

        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            conn_handle,
            StatementState::Allocated,
        ));
        validate_diag_rec(HandleType::Stmt, stmt_handle);
    }

    #[test]
    fn test_error_message() {
        use cstr::WideChar;
        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        // Initialize buffers
        let mut sql_state: [WideChar; 6] = [0; 6];
        let sql_state = &mut sql_state as *mut WideChar;
        let mut message_text: [WideChar; 57] = [0; 57];
        let message_text = &mut message_text as *mut WideChar;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;

        unsafe {
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
            // Buffer is too small to hold the entire error message and the null terminator
            // (0 < length < 57)
            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLGetDiagRecW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    15,
                    text_length_ptr
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
                SQLGetDiagRecW(
                    HandleType::Env,
                    env_handle as *mut _,
                    2,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    57,
                    text_length_ptr
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

        let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

        // Initialize buffers
        let mut sql_state: [WideChar; 6] = [0; 6];
        let sql_state = &mut sql_state as *mut WideChar;
        let mut message_text: [WideChar; 57] = [0; 57];
        let message_text = &mut message_text as *mut WideChar;
        let text_length_ptr = &mut 0;
        let native_err_ptr = &mut 0;

        unsafe {
            (*env_handle).add_diag_info(ODBCError::Unimplemented("SQLDrivers"));
            // Buffer length < 0
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagRecW(
                    HandleType::Env,
                    env_handle as *mut _,
                    1,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    -1,
                    text_length_ptr
                )
            );
            // Record number <= 0
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetDiagRecW(
                    HandleType::Env,
                    env_handle as *mut _,
                    0,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    57,
                    text_length_ptr
                )
            );
            // Record number > number of diagnostic records
            assert_eq!(
                SqlReturn::NO_DATA,
                SQLGetDiagRecW(
                    HandleType::Env,
                    env_handle as *mut _,
                    3,
                    sql_state,
                    native_err_ptr,
                    message_text,
                    5,
                    text_length_ptr
                )
            );
        }
    }
}
