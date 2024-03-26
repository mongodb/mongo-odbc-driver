mod common;

mod integration {
    use crate::common::{
        allocate_env, connect_and_allocate_statement, connect_with_conn_string,
        disconnect_and_close_handles,
    };
    use constants::DRIVER_NAME;
    use cstr::{to_char_ptr, to_widechar_ptr, WideChar};
    use definitions::{AttrOdbcVersion, SQLExecDirectW, SqlReturn};
    use lazy_static::lazy_static;
    use logger::Logger;
    use regex::Regex;
    use shared_sql_utils::driver_settings::{DriverSettings, LOGLEVEL, ODBCINSTINI};
    use shared_sql_utils::odbcinst::{SQLWritePrivateProfileString, SQLWritePrivateProfileStringW};
    use std::{fs, str, thread, time};

    #[test]
    fn test_invalid_connection() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        // Missing PWD
        let conn_str = "Driver=MongoDB Atlas SQL ODBC Driver;USER=N_A;SERVER=N_A";
        let result = connect_with_conn_string(env_handle, conn_str.to_string());

        assert!(
            result.is_err(),
            "The connection should have failed, but it was successful."
        );
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_default_connection() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str = crate::common::generate_default_connection_str();
        let _ = connect_with_conn_string(env_handle, conn_str).unwrap();
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn uuid_csharp_legacy() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str = crate::common::generate_uri_with_default_connection_string(
            "uuidRepresentation=csharpLegacy",
        );
        let _ = connect_with_conn_string(env_handle, conn_str).unwrap();
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn uuid_java_legacy() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str = crate::common::generate_uri_with_default_connection_string(
            "uuidRepresentation=javaLegacy",
        );
        let _ = connect_with_conn_string(env_handle, conn_str).unwrap();
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn uuid_python_legacy() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str = crate::common::generate_uri_with_default_connection_string(
            "uuidRepresentation=pythonLegacy",
        );
        let _ = connect_with_conn_string(env_handle, conn_str).unwrap();
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    /**
     * The following tests require a DSN called "ADF_Test" to be configured on the machine running the tests.
     */
    mod test_dsn {
        use crate::common::{allocate_env, connect_with_conn_string};
        use definitions::AttrOdbcVersion;

        #[test]
        fn test_valid_dsn_connection() {
            let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
            let conn_str = "DSN=ADF_Test";
            connect_with_conn_string(env_handle, conn_str.to_string()).unwrap();
            let _ = unsafe { Box::from_raw(env_handle) };
        }

        #[test]
        fn test_uri_opts_override_dsn() {
            let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
            let conn_str = "PWD=wrong;DSN=ADF_Test";
            let result = connect_with_conn_string(env_handle, conn_str.to_string());
            assert!(
                result.is_err(),
                "The connection should have failed, but it was successful."
            );
            let _ = unsafe { Box::from_raw(env_handle) };
        }
    }

    // Log level tests. Driver log level setting get overriden by connection log level //
    lazy_static! {
        static ref DEBUG_LINE: Regex = Regex::new(r"DEBUG: \[Env_0x([a-z0-9]+)\]\[Conn_0x([a-z0-9]+)\] SQLDriverConnectW:: SQLReturn = SUCCESS").unwrap();
        static ref INFO_LINE: Regex = Regex::new(r"INFO: \[Env_0x([a-z0-9]+)\]\[Conn_0x([a-z0-9]+)\] SQLAllocHandle:: SQLReturn = SUCCESS").unwrap();
        static ref ERROR_LINE: Regex = Regex::new(r"ERROR: \[Env_0x([a-z0-9]+)\]\[Conn_0x([a-z0-9]+)\]\[Stmt_0x([a-z0-9]+)\] SQLExecDirectW").unwrap();
    }

    // Test that log level are processed correctly.
    // By default, when no log level is specified, the log level is INFO
    // If a log level is set in the driver setting, this is the one used until connection.
    // At connection time, when the connection string is processed, if there is a log level specified
    // the logger log level is updated to the connection log level.
    #[test]
    fn test_driver_log_level() {
        let driver_settings: DriverSettings =
            DriverSettings::from_private_profile_string().unwrap_or_default();

        let log_dir = Logger::get_log_dir(driver_settings.driver.to_string());
        let log_file_path = log_dir.join("mongo_odbc.log");

        // Ensure we remove the log file if it exists. We don't care if it errors since
        // that means it doesn't exist (most likely), or we don't have permissions to touch
        // it, which will make this test very hard to run anyway!
        fs::remove_file(&log_file_path).unwrap_or_default();

        let log_file = log_file_path.as_os_str().to_str().unwrap().to_string();

        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);

        let mut conn_str = crate::common::generate_default_connection_str();
        let (dbc1, stmt1) = connect_and_allocate_statement(env_handle, Some(conn_str));

        disconnect_and_close_handles(dbc1, stmt1);

        // Wait a little to allow the logger to flush
        log::logger().flush();
        let one_sec = time::Duration::from_secs(1);
        thread::sleep(one_sec);

        let mut log_content = fs::read_to_string(&log_file).unwrap_or_default();

        // There should be no debug log line, only INFO because INFO is the default level for the driver
        assert!(INFO_LINE.is_match(&log_content));
        assert!(!DEBUG_LINE.is_match(&log_content));

        conn_str = crate::common::generate_default_connection_str();
        conn_str.push_str("LogLevel=Debug");
        let (dbc2, stmt2) = connect_and_allocate_statement(env_handle, Some(conn_str));

        disconnect_and_close_handles(dbc2, stmt2);

        // Wait a little to allow the logger to flush
        log::logger().flush();
        thread::sleep(one_sec);

        log_content = fs::read_to_string(&log_file).unwrap_or_default();

        // The connection log level took over, we should see Debug lines now
        assert!(DEBUG_LINE.is_match(&log_content));

        // Setting driver level to ERROR
        let error_log_level = "ERROR";
        write_driver_log_level(error_log_level);
        let driver_settings: DriverSettings =
            DriverSettings::from_private_profile_string().unwrap_or_default();
        assert_eq!(driver_settings.log_level, error_log_level);

        let conn_str = crate::common::generate_default_connection_str();
        let (dbc3, stmt3) = connect_and_allocate_statement(env_handle, Some(conn_str));
        // Execute an incorrect query to generate an Error log
        let mut query: Vec<WideChar> = cstr::to_widechar_vec("This is a pile of garbage");
        query.push(0);
        unsafe {
            // Only prepared statement can be executed.
            // Calling SQLExecute before SQLPrepare is invalid.
            assert_eq!(
                SqlReturn::ERROR,
                SQLExecDirectW(stmt3, query.as_ptr(), query.len() as i32)
            );
        }

        log::logger().flush();
        let log_content_error = fs::read_to_string(&log_file).unwrap_or_default();

        let original_log_size = log_content.len();
        let bytes = log_content_error.as_bytes();
        let new_content = &bytes[original_log_size..];
        let new_content_str = str::from_utf8(new_content).unwrap();

        // We should only see error logs show
        assert!(ERROR_LINE.is_match(new_content_str));
        assert!(!DEBUG_LINE.is_match(new_content_str));
        assert!(!INFO_LINE.is_match(new_content_str));

        // Clean-up
        fs::remove_file(log_file_path).unwrap_or_default();

        disconnect_and_close_handles(dbc3, stmt3);

        let _ = unsafe { Box::from_raw(env_handle) };

        let empty_log_level = "";
        write_driver_log_level(empty_log_level);
    }

    // Update the driver configuration and set the log level to the provided log level by either
    // writing to the odbinst ini file or to the registry.
    fn write_driver_log_level(log_level: &str) -> bool {
        unsafe {
            if cfg!(not(target_os = "linux")) {
                SQLWritePrivateProfileStringW(
                    to_widechar_ptr(DRIVER_NAME).0,
                    to_widechar_ptr(LOGLEVEL).0,
                    to_widechar_ptr(log_level).0,
                    to_widechar_ptr(ODBCINSTINI).0,
                )
            } else {
                SQLWritePrivateProfileString(
                    to_char_ptr(DRIVER_NAME).0,
                    to_char_ptr(LOGLEVEL).0,
                    to_char_ptr(log_level).0,
                    to_char_ptr(ODBCINSTINI).0,
                )
            }
        }
    }
}
