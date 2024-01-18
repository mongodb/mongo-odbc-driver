mod common;

mod integration {
    use crate::common::{
        allocate_env, connect_and_allocate_statement, disconnect_and_close_handles,
        fetch_and_get_data, get_column_attributes, get_sql_diagnostics,
    };
    use definitions::{
        CDataType, HStmt, Handle, HandleType, SQLExecute, SQLFetch, SQLPrepareW, SmallInt,
        SqlReturn, NTS,
    };

    use cstr::WideChar;

    #[test]
    fn test_error_execute_before_prepare() {
        let env_handle = allocate_env().unwrap();
        let (dbc, stmt) = connect_and_allocate_statement(env_handle, None);

        let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
        query.push(0);
        unsafe {
            // Only prepared statement can be executed.
            // Calling SQLExecute before SQLPrepare is invalid.
            assert_eq!(SqlReturn::ERROR, SQLExecute(stmt));

            let diagnostic = get_sql_diagnostics(HandleType::Stmt, stmt as Handle);
            // This error is thrown by the DM
            assert!(diagnostic.contains("Function sequence error"));
        }
        disconnect_and_close_handles(dbc, stmt);
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_prepare_get_resultset_metadata() {
        let env_handle = allocate_env().unwrap();
        let (dbc, stmt) = connect_and_allocate_statement(env_handle, None);

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            // Preparing a statement.
            // Only the result set metadata are retrieved and stored
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // Retrieve result set metadata
            get_column_attributes(stmt as Handle, 2);

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_error_fetch_before_execute() {
        let env_handle = allocate_env().unwrap();
        let (dbc, stmt) = connect_and_allocate_statement(env_handle, None);

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // Retrieve result set metadata
            get_column_attributes(stmt as Handle, 2);

            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt as HStmt),);

            let diagnostic = get_sql_diagnostics(HandleType::Stmt, stmt as Handle);
            assert!(diagnostic.contains("Function sequence error"));

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_prepare_execute_retrieve_data() {
        let env_handle = allocate_env().unwrap();
        let (dbc, stmt) = connect_and_allocate_statement(env_handle, None);

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // Retrieve result set metadata
            get_column_attributes(stmt as Handle, 2);

            // Executing the prepared statement.
            // The $sql pipeline is now executed and the result set cursor.
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_WCHAR],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    #[test]
    fn test_prepare_execute_multiple_times() {
        let env_handle = allocate_env().unwrap();
        let (dbc, stmt) = connect_and_allocate_statement(env_handle, None);

        unsafe {
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // Executing the prepared statement.
            // The $sql pipeline is now executed and the result set cursor.
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // Executing the prepared statement.
            // The $sql pipeline is now executed and the result set cursor.
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_WCHAR],
            );

            // A prepared statement can be executed multiple times.
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_WCHAR],
            );

            disconnect_and_close_handles(dbc, stmt);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }
}
