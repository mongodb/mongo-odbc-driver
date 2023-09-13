mod common;

mod integration {
    use crate::common::{
        allocate_env, allocate_statement, connect_with_conn_string, disconnect_and_close_handles,
        fetch_and_get_data, get_column_attributes, get_sql_diagnostics,
    };
    use odbc_sys::{
        CDataType, HDbc, HStmt, Handle, HandleType, SQLExecute, SQLFetch, SQLPrepareW, SmallInt,
        SqlReturn, NTS,
    };

    use cstr::WideChar;

    fn connect_and_allocate_statement() -> (HDbc, HStmt) {
        let env_handle = allocate_env().unwrap();
        let conn_str = crate::common::generate_default_connection_str();
        let conn_handle = connect_with_conn_string(env_handle, conn_str).unwrap();
        (conn_handle, allocate_statement(conn_handle).unwrap())
    }

    #[test]
    fn test_error_execute_before_prepare() {
        let (dbc, stmt) = connect_and_allocate_statement();

        let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
        query.push(0);
        unsafe {
            // Only prepared statement can be executed.
            // Calling SQLExecute before SQLPrepare is invalid.
            assert_eq!(
                SqlReturn::ERROR,
                SQLExecute(stmt)
            );

            dbg!(get_sql_diagnostics(HandleType::Stmt, stmt as Handle));
        }
        disconnect_and_close_handles(dbc, stmt);
    }

    #[test]
    fn test_prepare_get_resultset_metadata() {
        let (dbc, stmt) = connect_and_allocate_statement();

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
    }

    #[test]
    fn test_error_fetch_before_execute() {
        let (dbc, stmt) = connect_and_allocate_statement();

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

            assert_eq!(
                SqlReturn::ERROR,
                SQLFetch(stmt as HStmt),
            );

            dbg!(get_sql_diagnostics(HandleType::Stmt, stmt as Handle));

            disconnect_and_close_handles(dbc, stmt);
        }
    }

    #[test]
    fn test_prepare_execute_retrieve_data() {
        let (dbc, stmt) = connect_and_allocate_statement();

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
                vec![CDataType::SLong, CDataType::WChar],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            disconnect_and_close_handles(dbc, stmt);
        }
    }

    #[test]
    fn test_prepare_execute_multiple_times() {
        let (dbc, stmt) = connect_and_allocate_statement();

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

            fetch_and_get_data(
                stmt as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SLong, CDataType::WChar],
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
                vec![CDataType::SLong, CDataType::WChar],
            );

            disconnect_and_close_handles(dbc, stmt);
        }
    }
}
