mod common;

mod integration {

    use crate::common::{allocate_env, connect_with_conn_string};

    #[test]
    fn test_invalid_connection() {
        let env_handle = allocate_env().unwrap();
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
        let env_handle = allocate_env().unwrap();
        let conn_str = crate::common::generate_default_connection_str();
        let _ = connect_with_conn_string(env_handle, conn_str).unwrap();
        let _ = unsafe { Box::from_raw(env_handle) };
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

            let _ = fs::remove_file(&log_file_path);

            let env_handle = allocate_env().unwrap();
            let conn_str = format!(
                "{}{}",
                crate::common::generate_default_connection_str(),
                "loglevel=debug",
            );
            connect_with_conn_string(env_handle, conn_str).unwrap();
            let _ = unsafe { Box::from_raw(env_handle) };

            let log = fs::read_to_string(log_file_path).unwrap();

            assert!(log.contains("INFO:"));
            assert!(log.contains("DEBUG:"));
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
            let env_handle = allocate_env().unwrap();
            let conn_str = "DSN=ADF_Test";
            connect_with_conn_string(env_handle, conn_str.to_string()).unwrap();
            let _ = unsafe { Box::from_raw(env_handle) };
        }

        #[test]
        fn test_uri_opts_override_dsn() {
            let env_handle = allocate_env().unwrap();
            let conn_str = "PWD=wrong;DSN=ADF_Test";
            let result = connect_with_conn_string(env_handle, conn_str.to_string());
            assert!(
                result.is_err(),
                "The connection should have failed, but it was successful."
            );
            let _ = unsafe { Box::from_raw(env_handle) };
        }
    }
}
