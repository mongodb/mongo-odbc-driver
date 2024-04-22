mod common;

mod integration {
    use crate::common::{
        allocate_env, allocate_statement, connect_with_conn_string,
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, FreeStmtOption, Handle, HandleType, SQLExecDirectW, SQLFreeHandle,
        SQLFreeStmt, SqlReturn, SQL_NTS,
    };

    #[test]
    fn sql_disconnect_closes_statement_implicitly() {
        let (env, dbc, stmt) = default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);
        let query = b"SELECT * FROM integration_test.foo\0".map(|b| b as u16);
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt, query.as_ptr(), SQL_NTS as i32,),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );

            // we will not free the statement, instead proceeding directly to disconnect and freeing the dbc and env handles
            disconnect_and_free_dbc_and_env_handles(env, dbc)
        }
    }

    #[test]
    fn sql_disconnect_implicitly_closing_statment_cursors_is_safe_for_closed_statements() {
        let (env, dbc, stmt) = default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);
        let query = b"SELECT * FROM integration_test.foo\0".map(|b| b as u16);
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt, query.as_ptr(), SQL_NTS as i32,),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt, FreeStmtOption::SQL_CLOSE as i16),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );

            // we will not free the statement, instead proceeding directly to disconnect and freeing the dbc and env handles
            disconnect_and_free_dbc_and_env_handles(env, dbc)
        }
    }

    #[test]
    fn sql_disconnect_implicitly_closing_statment_cursors_is_safe_for_unbound_statemens() {
        let (env, dbc, stmt) = default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);
        let query = b"SELECT * FROM integration_test.foo\0".map(|b| b as u16);
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt, query.as_ptr(), SQL_NTS as i32,),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );

            // we didn't actually bind a statement, so this should be a no-op
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt, FreeStmtOption::SQL_UNBIND as i16),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );

            // we will not free the statement, instead proceeding directly to disconnect and freeing the dbc and env handles
            disconnect_and_free_dbc_and_env_handles(env, dbc)
        }
    }

    #[test]
    fn sql_disconnect_handles_many_statements_properly() {
        let env = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let dbc = connect_with_conn_string(env, None)
            .expect("Failed to connect with default connection string");
        let stmt_1 = allocate_statement(dbc).expect("Failed to allocate statement 1");
        let stmt_2 = allocate_statement(dbc).expect("Failed to allocate statement 2");
        let stmt_3 = allocate_statement(dbc).expect("Failed to allocate statement 3");
        let stmt_4 = allocate_statement(dbc).expect("Failed to allocate statement 4");
        let stmt_5 = allocate_statement(dbc).expect("Failed to allocate statement 5");
        let statements = [stmt_1, stmt_2, stmt_3, stmt_4, stmt_5];

        let query = b"SELECT * FROM integration_test.foo\0".map(|b| b as u16);
        statements.iter().for_each(|stmt| unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(*stmt, query.as_ptr(), SQL_NTS as i32,),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt.cast::<Handle>()),
            );
        });

        // we will close statements 1 and 3
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_1, FreeStmtOption::SQL_CLOSE as i16),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt_1.cast::<Handle>()),
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_3, FreeStmtOption::SQL_CLOSE as i16),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt_3.cast::<Handle>()),
            );

            // and we'll free statement 3 handle
            assert_eq!(
                SqlReturn::SUCCESS,
                // interestingly, rust doesn't like the cast here, so we'll just use the raw value
                SQLFreeHandle(HandleType::SQL_HANDLE_STMT as i16, stmt_3 as *mut _),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, *stmt_3.cast::<Handle>())
            );
        }

        disconnect_and_free_dbc_and_env_handles(env, dbc);
    }
}
