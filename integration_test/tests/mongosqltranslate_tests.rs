mod common;

mod mongosqltranslate {
    use crate::common::{
        allocate_env, connect_with_conn_string, default_setup_connect_and_alloc_stmt,
        disconnect_and_close_handles, fetch_and_get_data, get_column_attributes,
        get_sql_diagnostics,
    };
    use cstr::WideChar;
    use definitions::{
        AttrOdbcVersion, CDataType, HStmt, Handle, HandleType, SQLExecDirectW, SQLExecute,
        SQLPrepareW, SqlReturn, SQL_NTS,
    };

    #[test]
    fn test_srv_style_uri_connection() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let conn_str =
            crate::common::generate_srv_style_connection_string(Some("test".to_string()));
        let result = connect_with_conn_string(env_handle, Some(conn_str));

        assert!(
            result.is_ok(),
            "Expected successful connection, got error: {:?}",
            result
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
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT property_type, room_type, bed_type, minimum_nights, maximum_nights FROM listingsAndReviews LIMIT 3");
            query.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            get_column_attributes(stmt as Handle, 5);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 5],
                vec![CDataType::SQL_C_WCHAR; 5],
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
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("SELECT property_type, room_type, bed_type, minimum_nights, maximum_nights FROM listingsAndReviews LIMIT 3");
            query.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            get_column_attributes(stmt as Handle, 5);

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 5],
                vec![CDataType::SQL_C_WCHAR; 5],
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_enterprise_mode_with_library_loaded_and_invalid_query_and_valid_schemas_created() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "sample_airbnb".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> =
                cstr::to_widechar_vec("select * from non_existent_collection");
            query.push(0);

            assert_eq!(
                SqlReturn::ERROR,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let error_message = get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle);
            assert!(
                error_message.contains("No schema information returned for the requested collections."),
                "Expected error message: `No schema information returned for the requested collections.`; actual error message: {}",
                error_message
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_enterprise_mode_with_library_loaded_and_valid_query_and_no_sql_schemas_collection() {
        let (env_handle, dbc, stmt) = default_setup_connect_and_alloc_stmt(
            AttrOdbcVersion::SQL_OV_ODBC3,
            Some(crate::common::generate_srv_style_connection_string(Some(
                "test".to_string(),
            ))),
        );

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from foo");
            query.push(0);

            assert_eq!(
                SqlReturn::ERROR,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), SQL_NTS),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            let error_message = get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle);
            assert!(
                error_message.contains("The libmongosqltranslate command `translate` failed. Error message: `algebrize error: Error 1016: unknown collection 'foo' in database 'test'`. Error is internal: false"),
                "Expected error message: `The libmongosqltranslate command `translate` failed. Error message: `algebrize error: Error 1016: unknown collection 'foo' in database 'test'`. Error is internal: false`; actual error message: {}",
                error_message
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }
}
