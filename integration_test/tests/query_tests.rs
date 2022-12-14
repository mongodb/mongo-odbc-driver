mod common;

mod integration {

    #[test]
    fn test_odbc_query() {
        use odbc::*;
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
            _ => panic!("no data"),
        };
    }

    #[test]
    fn ffftest_columns() {
        use odbc_api::*;
        let mut conn_string = crate::common::generate_default_connection_str();
        conn_string.push_str("DATABASE=integration_test");

        let env = Environment::new().unwrap();
        let conn = env.connect_with_connection_string(&conn_string).unwrap();

        let mut cursor = conn.columns("", "", "", "");
        while let Ok(Some(mut row)) = cursor.as_mut().unwrap().next_row() {
            let mut buf = Vec::new();
            println!("NEXT");
            row.get_text(1, &mut buf).unwrap();
            println!("DB NAME: {}", std::str::from_utf8(&buf).unwrap());
            row.get_text(3, &mut buf).unwrap();
            println!("TABLE NAME: {}", std::str::from_utf8(&buf).unwrap());
            row.get_text(4, &mut buf).unwrap();
            println!("COL NAME: {}", std::str::from_utf8(&buf).unwrap());
        }
    }
}
