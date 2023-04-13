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

    /**
     * The following tests require a DSN called "ADF_Test" to be configured on the machine running the tests.
     */

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
