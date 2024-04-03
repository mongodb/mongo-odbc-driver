mod common;

macro_rules! test_connection_diagnostics {
    ($func_name:ident,
    in_connection_string = $in_connection_string:expr,
        driver_completion = $driver_completion:expr,
        expected_sql_state = $expected_sql_state:expr,
        expected_sql_return = $expected_sql_return:expr,
        expected_error_message = $expected_error_message:expr) => {
        #[test]
        fn $func_name() {
            use cstr::WideChar;
            use definitions::SmallInt;
            let in_connection_string = $in_connection_string;
            let driver_completion = $driver_completion;
            let expected_sql_state = $expected_sql_state;
            let expected_sql_return = $expected_sql_return;
            let expected_error_message = $expected_error_message;

            let mut out_connection_string: [WideChar; 64] = [0; 64];
            let out_connection_string = &mut out_connection_string as *mut WideChar;
            let string_length_2 = &mut 0;
            let buffer_length: SmallInt = 65;
            let mut env_handl: Handle = null_mut();
            let mut conn_handl: Handle = null_mut();

            let mut expected_sql_state_encoded =
                cstr::to_widechar_vec(expected_sql_state.odbc_3_state);
            expected_sql_state_encoded.push(0);
            let mut in_connection_string_encoded = cstr::to_widechar_vec(in_connection_string);
            in_connection_string_encoded.push(0);

            unsafe {
                let _ = SQLAllocHandle(
                    HandleType::SQL_HANDLE_ENV,
                    std::ptr::null_mut(),
                    &mut env_handl as *mut Handle,
                );
                let _ = SQLAllocHandle(
                    HandleType::SQL_HANDLE_DBC,
                    env_handl,
                    &mut conn_handl as *mut Handle,
                );
                let actual_return_val = SQLDriverConnectW(
                    conn_handl as *mut _,
                    std::ptr::null_mut(),
                    in_connection_string_encoded.as_ptr(),
                    in_connection_string.len().try_into().unwrap(),
                    out_connection_string,
                    buffer_length,
                    string_length_2,
                    driver_completion as u16,
                );
                assert_eq!(expected_sql_return, actual_return_val);

                verify_sql_diagnostics(
                    HandleType::SQL_HANDLE_DBC,
                    conn_handl as *mut _,
                    1,
                    expected_sql_state.odbc_3_state,
                    expected_error_message,
                    0,
                );
                let _ = SQLFreeHandle(HandleType::SQL_HANDLE_DBC, conn_handl);
                let _ = SQLFreeHandle(HandleType::SQL_HANDLE_ENV, env_handl);
            };
        }
    };
}

mod integration {
    use crate::common::verify_sql_diagnostics;
    use atsql::{SQLAllocHandle, SQLDriverConnectW, SQLFreeHandle};
    use constants::{NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, UNABLE_TO_CONNECT};
    use definitions::{DriverConnectOption, Handle, HandleType, SqlReturn};
    use std::ptr::null_mut;

    test_connection_diagnostics! (
            missing_user_in_connection_string,
            in_connection_string = "Driver=MongoDB Atlas SQL ODBC Driver;SERVER=N_A;PWD=N_A",
            driver_completion = DriverConnectOption::SQL_DRIVER_NO_PROMPT,
            expected_sql_state = UNABLE_TO_CONNECT,
            expected_sql_return = SqlReturn::ERROR,
            expected_error_message = "[MongoDB][Core] Invalid Uri: One of [\"uid\", \"user\"] is required for a valid Mongo ODBC Uri"
        );
    test_connection_diagnostics! (
            missing_pwd_in_connection_string,
            in_connection_string = "Driver=MongoDB Atlas SQL ODBC Driver;SERVER=N_A;USER=N_A",
            driver_completion = DriverConnectOption::SQL_DRIVER_NO_PROMPT,
            expected_sql_state = UNABLE_TO_CONNECT,
            expected_sql_return = SqlReturn::ERROR,
            expected_error_message = "[MongoDB][Core] Invalid Uri: One of [\"password\", \"pwd\"] is required for a valid Mongo ODBC Uri"
        );
    test_connection_diagnostics!(
        missing_driver_in_connection_string,
        in_connection_string = "USER=N_A;SERVER=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::SQL_DRIVER_NO_PROMPT,
        expected_sql_state = NO_DSN_OR_DRIVER,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] Missing property \"Driver\" or \"DSN\" in connection string"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_prompt,
        in_connection_string = "USER=N_A;SERVER=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::SQL_DRIVER_PROMPT,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] The driver connect option SQL_DRIVER_PROMPT is not supported"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_complete,
        in_connection_string = "USER=N_A;SERVER=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::SQL_DRIVER_COMPLETE,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] The driver connect option SQL_DRIVER_COMPLETE is not supported"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_complete_required,
        in_connection_string = "USER=N_A;SERVER=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::SQL_DRIVER_COMPLETE_REQUIRED,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] The driver connect option SQL_DRIVER_COMPLETE_REQUIRED is not supported"
    );
}
