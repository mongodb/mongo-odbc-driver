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
            let env_handle: *mut MongoHandle =
                &mut MongoHandle::Env(RwLock::new(Env::with_state(EnvState::Allocated)));
            let conn_handle: *mut MongoHandle = &mut MongoHandle::Connection(RwLock::new(
                Connection::with_state(env_handle, ConnectionState::Allocated),
            ));
            let mut expected_sql_state_encoded: Vec<u16> =
                expected_sql_state.encode_utf16().collect();
            expected_sql_state_encoded.push(0);
            let mut in_connection_string_encoded: Vec<u16> =
                in_connection_string.encode_utf16().collect();
            in_connection_string_encoded.push(0);

            let actual_return_val = SQLDriverConnectW(
                conn_handle as *mut _,
                std::ptr::null_mut(),
                in_connection_string_encoded.as_ptr(),
                in_connection_string.len().try_into().unwrap(),
                out_connection_string,
                buffer_length,
                string_length_2,
                driver_completion,
            );
            assert_eq!(expected_sql_return, actual_return_val);

            let text_length_ptr = &mut 0;
            let actual_sql_state = &mut [0u16; 6] as *mut _;
            let actual_message_text = &mut [0u16; 512] as *mut _;
            // Using SQLGetDiagRecW to get the sql state and error message
            // from the connection handle
            let _ = SQLGetDiagRecW(
                HandleType::Dbc,
                conn_handle as *mut _,
                1,
                actual_sql_state,
                &mut 0,
                actual_message_text,
                512,
                text_length_ptr,
            );
            let actual_message_length = *text_length_ptr as usize;
            unsafe {
                assert_eq!(
                    expected_error_message,
                    &(String::from_utf16_lossy(&*(actual_message_text as *const [u16; 256])))
                        [0..actual_message_length],
                );
                assert_eq!(
                    String::from_utf16(&*(expected_sql_state_encoded.as_ptr() as *const [u16; 6]))
                        .unwrap(),
                    String::from_utf16(&*(actual_sql_state as *const [u16; 6])).unwrap()
                );
            }
        }
    };
}

mod unit {
    use crate::{handles::definitions::*, SQLDriverConnectW, SQLGetDiagRecW};
    use constants::{NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, UNABLE_TO_CONNECT};
    use odbc_sys::DriverConnectOption;
    use odbc_sys::{HandleType, SqlReturn};
    use std::sync::RwLock;

    test_connection_diagnostics! (
            illegal_character_in_server_field,
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
