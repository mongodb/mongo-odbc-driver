mod common;

mod integration {

    #[test]
    fn query() {
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
            _ => panic!("No data found in Collection `example`. Make sure that the integration test environment is setup correctly."),
        };
    }

    fn columns_test(expected: &[[&str; 3]], filters: (&str, &str, &str)) {
        use odbc_api::*;
        let mut conn_string = crate::common::generate_default_connection_str();
        conn_string.push_str("DATABASE=integration_test");

        let env = Environment::new().unwrap();
        let conn = env.connect_with_connection_string(&conn_string).unwrap();

        let mut cursor = conn.columns(filters.0, "", filters.1, filters.2);
        let mut i = 0;
        while let Ok(Some(mut row)) = cursor.as_mut().unwrap().next_row() {
            let mut buf = Vec::new();
            let expected_row = expected[i];
            i += 1;
            row.get_text(1, &mut buf).unwrap();
            assert_eq!(expected_row[0], std::str::from_utf8(&buf).unwrap());
            row.get_text(3, &mut buf).unwrap();
            assert_eq!(expected_row[1], std::str::from_utf8(&buf).unwrap());
            row.get_text(4, &mut buf).unwrap();
            assert_eq!(expected_row[2], std::str::from_utf8(&buf).unwrap());
        }
        // assert that there were actually the correct number of rows in the resultset.
        assert_eq!(
            expected.len(),
            i,
            "Expected {} rows, found {}",
            expected.len(),
            i
        );
    }

    #[test]
    fn all_columns() {
        let expected = [
            ["integration_test", "example", "_id"],
            ["integration_test", "example", "b"],
            ["integration_test", "foo", "_id"],
            ["integration_test", "foo", "a"],
            ["integration_test_2", "example_2", "_id"],
            ["integration_test_2", "example_2", "b"],
        ];
        columns_test(&expected, ("", "", ""));
    }

    #[test]
    fn columns_with_column_filter() {
        let expected = [
            ["integration_test", "example", "_id"],
            ["integration_test", "foo", "_id"],
            ["integration_test_2", "example_2", "_id"],
        ];
        columns_test(&expected, ("", "", "%i%"));
    }

    #[test]
    fn columns_with_collection_filter() {
        let expected = [
            ["integration_test", "example", "_id"],
            ["integration_test", "example", "b"],
            ["integration_test_2", "example_2", "_id"],
            ["integration_test_2", "example_2", "b"],
        ];
        columns_test(&expected, ("", "%mp%", ""));
    }

    #[test]
    fn columns_with_catalog() {
        let expected = [
            ["integration_test_2", "example_2", "_id"],
            ["integration_test_2", "example_2", "b"],
        ];
        columns_test(&expected, ("integration_test_2", "", ""));
    }

    #[test]
    fn columns_with_all_filters() {
        let expected = [["integration_test", "foo", "_id"]];
        columns_test(&expected, ("integration_test", "%o%", "%i%"));
    }

    // TODO SQL-1155:
    //    #[test]
    //    fn columns_with_schema_is_error() {
    //        use odbc_api::*;
    //
    //        let mut conn_string = crate::common::generate_default_connection_str();
    //        conn_string.push_str("DATABASE=integration_test");
    //
    //        let env = Environment::new().unwrap();
    //        let conn = env.connect_with_connection_string(&conn_string).unwrap();
    //
    //        let res = conn.columns("", "foo", "", "");
    //        assert!(res.is_err());
    //        unsafe {
    //            assert_eq!(
    //                "ODBC emitted an error calling 'SQLColumns':\nState: HYC00, Native error: 0, Message: ".to_string(),
    //                // have to use unchecked because the library implementors very nicely
    //                // decided not to derive Debug. EVERYTHING should just derive Debug.
    //                format!("{}", res.unwrap_err_unchecked()),
    //            );
    //        }
    //    }
}
