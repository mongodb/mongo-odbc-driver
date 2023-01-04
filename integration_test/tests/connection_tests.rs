mod common;

mod integration {
    use odbc::create_environment_v3;

    #[test]
    fn test_invalid_connection() {
        let env = create_environment_v3().unwrap();
        // Missing PWD
        let conn_str = "Driver=ADF_ODBC_DRIVER;USER=N_A;SERVER=N_A";
        let result = env.connect_with_connection_string(conn_str);
        assert!(
            result.is_err(),
            "The connection should have failed, but it was successful."
        );
    }

    #[test]
    fn test_default_connection() {
        let env = create_environment_v3().unwrap();
        let conn_str = crate::common::generate_default_connection_str();
        env.connect_with_connection_string(&conn_str).unwrap();
    }
}
