mod common;

mod mongosqltranslate_tests {
    use crate::common::{
        allocate_env, connect_with_conn_string, default_setup_connect_and_alloc_stmt,
        disconnect_and_close_handles, fetch_and_get_data, get_column_attributes,
        get_sql_diagnostics,
    };
    use cstr::WideChar;
    use definitions::{
        AttrOdbcVersion, CDataType, HStmt, Handle, HandleType, SQLColumnsW, SQLExecDirectW,
        SQLExecute, SQLPrepareW, SmallInt, SqlReturn, SQL_NTS,
    };
    use serde_json::{Number, Value};
    use std::ptr;

    #[test]
    fn test_srv_style_uri_connection() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str =
            crate::common::generate_srv_style_connection_string(Some("test".to_string()));
        let result = connect_with_conn_string(env_handle, Some(conn_str), true);

        assert!(
            result.is_ok(),
            "Expected successful connection, got error: {result:?}"
        );

        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_sql_prepare_and_sql_execute_with_library_loaded_and_valid_query_and_valid_schemas_created(
    ) {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "sample_airbnb".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT property_type, room_type, bed_type, minimum_nights, maximum_nights FROM listingsAndReviews ORDER BY _id LIMIT 3");
            query.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let expected_column_metadata_values = create_expected_column_metadata();

            get_column_attributes(stmt as Handle, 5, Some(expected_column_metadata_values));

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let expected_column_values = create_expected_column_values();

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 5],
                vec![CDataType::SQL_C_WCHAR; 5],
                Some(expected_column_values),
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_sql_execute_direct_with_library_loaded_and_valid_query_and_valid_schemas_created() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "sample_airbnb".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT property_type, room_type, bed_type, minimum_nights, maximum_nights FROM listingsAndReviews ORDER BY _id LIMIT 3");
            query.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let expected_column_metadata_values = create_expected_column_metadata();

            get_column_attributes(stmt as Handle, 5, Some(expected_column_metadata_values));

            let expected_column_values = create_expected_column_values();

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 5],
                vec![CDataType::SQL_C_WCHAR; 5],
                Some(expected_column_values),
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_enterprise_mode_with_invalid_query_and_valid_schemas_created() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "sample_airbnb".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> =
                cstr::to_widechar_vec("SELECT bed_type.type FROM listingsAndReviews");
            query.push(0);

            assert_eq!(
                SqlReturn::ERROR,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let error_message = get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle);
            assert!(
                error_message.contains("[MongoDB][Core] The mongosql translate command `translate_sql` failed. Error message: `algebrize error: Error 1002: Incorrect argument type for `FieldAccess`. Required: object type. Found: string."),
                "Expected error message: `[MongoDB][Core] The mongosql translate command `translate_sql` failed. Error message: `algebrize error: Error 1002: Incorrect argument type for `FieldAccess`. Required: object type. Found: string.`; actual error message: {error_message}"
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_enterprise_mode_with_valid_query_and_no_sql_schemas_collection() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "test".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT * FROM foo");
            query.push(0);

            assert_eq!(
                SqlReturn::ERROR,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let error_message = get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle);
            assert!(
                error_message.contains("[MongoDB][Core] The mongosql translate command `translate_sql` failed. Error message: `algebrize error: Error 1016: unknown collection 'foo' in database 'test'`"),
                "Expected error message: `[MongoDB][Core] The mongosql translate command `translate_sql` failed. Error message: `algebrize error: Error 1016: unknown collection 'foo' in database 'test'`.; actual error message: {error_message}"
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_sql_columnsw_with_library_loaded_and_valid_schemas_created() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "sample_airbnb".to_string(),
            ))),
        );

        unsafe {
            let mut table_name: Vec<WideChar> = cstr::to_widechar_vec("listingsAndReviews");
            table_name.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLColumnsW(
                    stmt as HStmt,
                    ptr::null(),
                    0,
                    ptr::null(),
                    0,
                    table_name.as_ptr(),
                    SQL_NTS as SmallInt,
                    ptr::null(),
                    0
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            get_column_attributes(stmt as Handle, 18, None);

            let expected_rs_values = create_expected_sql_columnsw_rs_values();

            fetch_and_get_data(
                stmt as Handle,
                Some(42),
                vec![SqlReturn::SUCCESS; 6],
                vec![
                    CDataType::SQL_C_WCHAR,
                    CDataType::SQL_C_WCHAR,
                    CDataType::SQL_C_WCHAR,
                    CDataType::SQL_C_WCHAR,
                    CDataType::SQL_C_SLONG,
                    CDataType::SQL_C_WCHAR,
                ],
                Some(expected_rs_values),
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    fn create_expected_column_metadata() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::Number(Number::from(-9)),
                Value::Number(Number::from(1)),
                Value::String("property_type".to_string()),
                Value::Number(Number::from(0)),
                Value::String("string".to_string()),
                Value::Number(Number::from(0)),
                Value::Number(Number::from(0)),
            ],
            vec![
                Value::Number(Number::from(-9)),
                Value::Number(Number::from(1)),
                Value::String("room_type".to_string()),
                Value::Number(Number::from(0)),
                Value::String("string".to_string()),
                Value::Number(Number::from(0)),
                Value::Number(Number::from(0)),
            ],
            vec![
                Value::Number(Number::from(-9)),
                Value::Number(Number::from(1)),
                Value::String("bed_type".to_string()),
                Value::Number(Number::from(0)),
                Value::String("string".to_string()),
                Value::Number(Number::from(0)),
                Value::Number(Number::from(0)),
            ],
            vec![
                Value::Number(Number::from(-9)),
                Value::Number(Number::from(1)),
                Value::String("minimum_nights".to_string()),
                Value::Number(Number::from(0)),
                Value::String("string".to_string()),
                Value::Number(Number::from(0)),
                Value::Number(Number::from(0)),
            ],
            vec![
                Value::Number(Number::from(-9)),
                Value::Number(Number::from(1)),
                Value::String("maximum_nights".to_string()),
                Value::Number(Number::from(0)),
                Value::String("string".to_string()),
                Value::Number(Number::from(0)),
                Value::Number(Number::from(0)),
            ],
        ]
    }

    fn create_expected_column_values() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::String("House".to_string()),
                Value::String("Entire home/apt".to_string()),
                Value::String("Real Bed".to_string()),
                Value::String("2".to_string()),
                Value::String("30".to_string()),
            ],
            vec![
                Value::String("Apartment".to_string()),
                Value::String("Entire home/apt".to_string()),
                Value::String("Real Bed".to_string()),
                Value::String("2".to_string()),
                Value::String("1125".to_string()),
            ],
            vec![
                Value::String("Condominium".to_string()),
                Value::String("Entire home/apt".to_string()),
                Value::String("Real Bed".to_string()),
                Value::String("3".to_string()),
                Value::String("365".to_string()),
            ],
        ]
    }

    fn create_expected_sql_columnsw_rs_values() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("_id".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("access".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("accommodates".to_string()),
                Value::Number(Number::from(4)),
                Value::String("int".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("address".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("object".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("amenities".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("array".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("availability".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("object".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("bathrooms".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("bed_type".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("bedrooms".to_string()),
                Value::Number(Number::from(4)),
                Value::String("int".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("beds".to_string()),
                Value::Number(Number::from(4)),
                Value::String("int".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("calendar_last_scraped".to_string()),
                Value::Number(Number::from(11)),
                Value::String("date".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("cancellation_policy".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("cleaning_fee".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("description".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("extra_people".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("first_review".to_string()),
                Value::Number(Number::from(11)),
                Value::String("date".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("guests_included".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("host".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("object".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("house_rules".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("images".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("object".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("interaction".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("last_review".to_string()),
                Value::Number(Number::from(11)),
                Value::String("date".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("last_scraped".to_string()),
                Value::Number(Number::from(11)),
                Value::String("date".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("listing_url".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("maximum_nights".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("minimum_nights".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("monthly_price".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("name".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("neighborhood_overview".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("notes".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("number_of_reviews".to_string()),
                Value::Number(Number::from(4)),
                Value::String("int".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("price".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("property_type".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("review_scores".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("object".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("reviews".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("array".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("reviews_per_month".to_string()),
                Value::Number(Number::from(4)),
                Value::String("int".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("room_type".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("security_deposit".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("space".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("summary".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("transit".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("string".to_string()),
            ],
            vec![
                Value::String("sample_airbnb".to_string()),
                Value::Null,
                Value::String("listingsAndReviews".to_string()),
                Value::String("weekly_price".to_string()),
                Value::Number(Number::from(-9)),
                Value::String("decimal".to_string()),
            ],
        ]
    }
}
