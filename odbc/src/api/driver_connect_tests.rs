use crate::{handles::definitions::*, SQLDriverConnectW, SQLGetDiagRecW};
use constants::{NOT_IMPLEMENTED, NO_DSN_OR_DRIVER, UNABLE_TO_CONNECT};
use odbc_sys::{DriverConnectOption, HandleType, SqlReturn};
use std::sync::RwLock;

mod unit {
    use super::*;
    use odbc_sys::SmallInt;
    #[test]
    fn connect_edge_cases() {
        fn driver_connect_check_diagnostics(
            test_name: &str,
            in_connection_string: &str,
            driver_completion: DriverConnectOption,
            expected_sql_state: &str,
            expected_sql_return: SqlReturn,
            expected_error_message: &str,
        ) {
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
                    "Failed test: {}",
                    test_name
                );
                assert_eq!(
                    *(expected_sql_state_encoded.as_ptr() as *const [u16; 6]),
                    *(actual_sql_state as *const [u16; 6]),
                    "Failed test: {}",
                    test_name
                );
            }
        }

        let in_connection_string =
            "Driver=ADF_ODBC_DRIVER;USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A";
        // Parse error due to illegal character in SERVER field
        driver_connect_check_diagnostics(
            "Illegal character in SERVER field",
            "PWD=N_A;Driver=ADF_ODBC_DRIVER;SERVER=//;AUTH_SRC=N_A;USER=N_A",
            DriverConnectOption::NoPrompt,
            UNABLE_TO_CONNECT,
            SqlReturn::ERROR,
            "[MongoDB][Core] Invalid connection string. Parse error: An invalid argument was provided: illegal character in database name",
        );

        // Missing Parameters in connection string
        // Missing 'USER'
        driver_connect_check_diagnostics(
            "Missing 'USER' parameter in connection string",
            "Driver=ADF_ODBC_DRIVER;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
            DriverConnectOption::NoPrompt,
            UNABLE_TO_CONNECT,
            SqlReturn::ERROR,
            "[MongoDB][API] Invalid Uri One of [\"user\", \"uid\"] is required for a valid Mongo ODBC Uri",
        );

        // Missing 'PWD'
        driver_connect_check_diagnostics(
            "Missing 'PWD' parameter in connection string",
            "Driver=ADF_ODBC_DRIVER;SERVER=N_A;AUTH_SRC=N_A;USER=N_A",
            DriverConnectOption::NoPrompt,
            UNABLE_TO_CONNECT,
            SqlReturn::ERROR,
            "[MongoDB][API] Invalid Uri One of [\"pwd\", \"password\"] is required for a valid Mongo ODBC Uri",
        );
        // Missing 'DRIVER'
        driver_connect_check_diagnostics(
            "Missing 'DRIVER' parameter in connection string",
            "USER=N_A;SERVER=N_A;AUTH_SRC=N_A;PWD=N_A",
            DriverConnectOption::NoPrompt,
            NO_DSN_OR_DRIVER,
            SqlReturn::ERROR,
            "[MongoDB][API] Missing property \"Driver\" or \"DSN\" in connection string",
        );

        // Unsupported Driver Completion options
        // DriverConnectOption::Prompt
        driver_connect_check_diagnostics(
            "Unsupported Driver Completion option: DriverConnectOption::Prompt",
            in_connection_string,
            DriverConnectOption::Prompt,
            NOT_IMPLEMENTED,
            SqlReturn::ERROR,
            "[MongoDB][API] The driver connect option Prompt is not supported",
        );
        // DriverConnectOption::Complete
        driver_connect_check_diagnostics(
            "Unsupported Driver Completion option: DriverConnectOption::Complete",
            in_connection_string,
            DriverConnectOption::Complete,
            NOT_IMPLEMENTED,
            SqlReturn::ERROR,
            "[MongoDB][API] The driver connect option Complete is not supported",
        );
        // DriverConnectOption::CompleteRequired
        driver_connect_check_diagnostics(
            "Unsupported Driver Completion option: DriverConnectOption::CompleteRequired",
            in_connection_string,
            DriverConnectOption::CompleteRequired,
            NOT_IMPLEMENTED,
            SqlReturn::ERROR,
            "[MongoDB][API] The driver connect option CompleteRequired is not supported",
        );
    }
}
