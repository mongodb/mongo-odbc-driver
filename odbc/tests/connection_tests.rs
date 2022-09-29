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
            use odbc_sys::SmallInt;
            let in_connection_string = $in_connection_string;
            let driver_completion = $driver_completion;
            let expected_sql_state = $expected_sql_state;
            let expected_sql_return = $expected_sql_return;
            let expected_error_message = $expected_error_message;

            let out_connection_string = &mut [0u16; 64] as *mut _;
            let string_length_2 = &mut 0;
            let buffer_length: SmallInt = 65;
            let mut env_handl: Handle = null_mut();
            let mut conn_handl: Handle = null_mut();

            let mut expected_sql_state_encoded: Vec<u16> =
                expected_sql_state.encode_utf16().collect();
            expected_sql_state_encoded.push(0);
            let mut in_connection_string_encoded: Vec<u16> =
                in_connection_string.encode_utf16().collect();
            in_connection_string_encoded.push(0);

            let _ = SQLAllocHandle(
                HandleType::Env,
                std::ptr::null_mut(),
                &mut env_handl as *mut Handle,
            );
            let _ = SQLAllocHandle(HandleType::Dbc, env_handl, &mut conn_handl as *mut Handle);
            let actual_return_val = SQLDriverConnectW(
                conn_handl as *mut _,
                std::ptr::null_mut(),
                in_connection_string_encoded.as_ptr(),
                in_connection_string.len().try_into().unwrap(),
                out_connection_string,
                buffer_length,
                string_length_2,
                driver_completion,
            );
            assert_eq!(expected_sql_return, actual_return_val);

            verify_sql_diagnostics(
                HandleType::Dbc,
                conn_handl as *mut _,
                1,
                expected_sql_state,
                expected_error_message,
                0,
            );
        }
    };
}

mod integration {
    use crate::common::{connect, verify_sql_diagnostics};
    use constants::{NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, UNABLE_TO_CONNECT};
    use mongoodbc::{SQLAllocHandle, SQLDriverConnectW};
    use odbc_sys::{DriverConnectOption, Handle, HandleType, SqlReturn};
    use std::ptr::null_mut;

    #[test]
    fn test_invalid_connection() {
        // Missing PWD
        let conn_str = "Driver=ADF_ODBC_DRIVER;USER=N_A;SERVER=N_A;AUTH_SRC=N_A";
        let result = connect(Some(conn_str));
        assert!(
            result.is_err(),
            "The connection should have failed, but it was successful."
        );
    }

    #[test]
    fn test_default_connection() {
        connect(None).unwrap();
    }

    test_connection_diagnostics! (
            invalid_connection_string_parse_error,
            in_connection_string = "PWD=N_A;Driver=ADF_ODBC_DRIVER;SERVER=//;AUTH_SRC=N_A;USER=N_A",
            driver_completion = DriverConnectOption::NoPrompt,
            expected_sql_state = UNABLE_TO_CONNECT,
            expected_sql_return = SqlReturn::ERROR,
            expected_error_message = "[MongoDB][Core] Invalid connection string. Parse error: An invalid argument was provided: illegal character in database name"
        );
    test_connection_diagnostics! (
            missing_user_in_connection_string,
            in_connection_string = "Driver=ADF_ODBC_DRIVER;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
            driver_completion = DriverConnectOption::NoPrompt,
            expected_sql_state = UNABLE_TO_CONNECT,
            expected_sql_return = SqlReturn::ERROR,
            expected_error_message = "[MongoDB][API] Invalid Uri: One of [\"user\", \"uid\"] is required for a valid Mongo ODBC Uri"
        );
    test_connection_diagnostics! (
            missing_pwd_in_connection_string,
            in_connection_string = "Driver=ADF_ODBC_DRIVER;SERVER=N_A;AUTH_SRC=N_A;USER=N_A",
            driver_completion = DriverConnectOption::NoPrompt,
            expected_sql_state = UNABLE_TO_CONNECT,
            expected_sql_return = SqlReturn::ERROR,
            expected_error_message = "[MongoDB][API] Invalid Uri: One of [\"pwd\", \"password\"] is required for a valid Mongo ODBC Uri"
        );
    test_connection_diagnostics!(
        missing_driver_in_connection_string,
        in_connection_string = "USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::NoPrompt,
        expected_sql_state = NO_DSN_OR_DRIVER,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] Missing property \"Driver\" or \"DSN\" in connection string"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_prompt,
        in_connection_string = "USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::Prompt,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message = "[MongoDB][API] The driver connect option Prompt is not supported"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_complete,
        in_connection_string = "USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::Complete,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] The driver connect option Complete is not supported"
    );
    test_connection_diagnostics!(
        unsupported_driver_connect_option_complete_required,
        in_connection_string = "USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
        driver_completion = DriverConnectOption::CompleteRequired,
        expected_sql_state = NOT_IMPLEMENTED,
        expected_sql_return = SqlReturn::ERROR,
        expected_error_message =
            "[MongoDB][API] The driver connect option CompleteRequired is not supported"
    );
}
