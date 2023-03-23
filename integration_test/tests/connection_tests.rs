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
}
