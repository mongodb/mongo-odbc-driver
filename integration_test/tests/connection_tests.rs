mod common;

mod integration {

    use crate::common::{allocate_env, connect_with_conn_string};

    #[test]
    fn test_invalid_connection() {
        let env_handle = allocate_env();
        // Missing PWD
        let conn_str = "Driver=MongoDB Atlas SQL ODBC Driver;USER=N_A;SERVER=N_A";
        let result = connect_with_conn_string(env_handle.unwrap(), conn_str.to_string());

        assert!(
            result.is_err(),
            "The connection should have failed, but it was successful."
        );
    }

    #[test]
    fn test_default_connection() {
        let env_handle = allocate_env();
        let conn_str = crate::common::generate_default_connection_str();
        let _ = connect_with_conn_string(env_handle.unwrap(), conn_str).unwrap();
    }

    #[cfg(test)]
    mod logs {
        use crate::common::{allocate_env, connect_with_conn_string};
        use constants::DRIVER_NAME;
        use cstr::to_widechar_ptr;
        use shared_sql_utils::odbcinst::SQLGetPrivateProfileStringW;

        #[test]
        fn test_logs_exist() {
            use std::{fs, path::Path};

            let env_handle = allocate_env();
            let conn_str = format!(
                "{}{}",
                crate::common::generate_default_connection_str(),
                "loglevel=debug",
            );
            let result = connect_with_conn_string(env_handle.unwrap(), conn_str).unwrap();
            let mut buffer = [0u16; 1024];
            unsafe {
                SQLGetPrivateProfileStringW(
                    to_widechar_ptr(DRIVER_NAME).0,
                    to_widechar_ptr("Driver").0,
                    to_widechar_ptr("").0,
                    buffer.as_mut_ptr(),
                    buffer.len() as i32,
                    to_widechar_ptr("odbcinst.ini").0,
                )
            };

            let driver_path = unsafe { cstr::parse_attribute_string_w(buffer.as_mut_ptr()) };
            let log_file_path = Path::new(&driver_path)
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(Path::new("logs").join(Path::new("mongo_odbc.log")))
                .to_str()
                .unwrap()
                .to_string();

            let log = fs::read_to_string(log_file_path).unwrap();

            assert!(log.contains("INFO:"));
            assert!(log.contains("DEBUG:"));
            let _ = result;
        }
    }

    /**
     * The following tests require a DSN called "ADF_Test" to be configured on the machine running the tests.
     */

    #[cfg(target_os = "windows")]
    mod test_dsn {
        use crate::common::{allocate_env, connect_with_conn_string};
        #[test]
        fn test_valid_dsn_connection() {
            let env_handle = allocate_env();
            let conn_str = "DSN=ADF_Test";
            connect_with_conn_string(env_handle.unwrap(), conn_str.to_string()).unwrap();
        }

        #[test]
        fn test_uri_opts_override_dsn() {
            let env_handle = allocate_env();
            let conn_str = "PWD=wrong;DSN=ADF_Test";
            let result = connect_with_conn_string(env_handle.unwrap(), conn_str.to_string());
            assert!(
                result.is_err(),
                "The connection should have failed, but it was successful."
            );
        }
    }
}
