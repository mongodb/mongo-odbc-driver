mod common;

mod integration {
    use super::common::connect;
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
}
