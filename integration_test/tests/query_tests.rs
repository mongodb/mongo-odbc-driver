mod common;

mod integration {
    use odbc::*;

    #[test]
    fn test_odbc_query() {
        let env = create_environment_v3().unwrap();
        let mut conn_string = crate::common::generate_default_connection_str();
        conn_string.push_str("DATABASE=integration_test");

        let conn = env.connect_with_connection_string(&conn_string);

        let conn = conn.unwrap();
        let stmt = Statement::with_parent(&conn).unwrap();

        let expected = ["a", "b", "c"];
        let mut i = 0;
        match stmt.exec_direct("SELECT * FROM example").unwrap() {
            Data(mut stmt) => {
                while let Some(mut cursor) = stmt.fetch().unwrap() {
                    match cursor.get_data::<&str>(2).unwrap() {
                        Some(val) => assert_eq!(expected[i], val),
                        _ => panic!("get_data failed"),
                    }
                    i += 1;
                }
            }
            _ => panic!("No data found in Collection `example`. Make sure that the integration test environment is setup correctly."),
        };
    }
}
