mod common;

mod integration {
    use std::ptr;

    use crate::common::{
        allocate_env, allocate_statement, connect_with_conn_string,
        disconnect_and_free_dbc_and_env_handles, get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, FreeStmtOption, Handle, HandleType, SQLDisconnect, SQLExecDirectW,
        SQLFreeHandle, SQLFreeStmt, SqlReturn, SQL_NTS,
    };

    #[test]
    fn sql_disconnect_handles_many_statement_states_properly() {
        let env = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        let dbc = connect_with_conn_string(env, None)
            .expect("Failed to connect with default connection string");
        // The following statements will be allocated and interacted with in the following ways:
        // stmt_1 will be closed
        let stmt_1 = allocate_statement(dbc).expect("Failed to allocate statement 1");
        // stmt_2 will be unbound
        let stmt_2 = allocate_statement(dbc).expect("Failed to allocate statement 2");
        // stmt_3 will have its handle deallocated
        let stmt_3 = allocate_statement(dbc).expect("Failed to allocate statement 3");
        // stmt_4 will have no operations performed on it other than a query
        let stmt_4 = allocate_statement(dbc).expect("Failed to allocate statement 4");
        let statements = [stmt_1, stmt_2, stmt_3, stmt_4];

        let query = b"SELECT * FROM integration_test.foo\0".map(|b| b as u16);
        // issue a query on each statement, resulting in an open cursor on each
        statements.iter().for_each(|stmt| unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(*stmt, query.as_ptr(), SQL_NTS as i32,),
                "{}",
                get_sql_diagnostics(
                    HandleType::SQL_HANDLE_STMT,
                    *ptr::addr_of!(stmt).cast::<Handle>()
                ),
            );
        });

        unsafe {
            // Close statement 1
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_1, FreeStmtOption::SQL_CLOSE as i16),
                "{}",
                get_sql_diagnostics(
                    HandleType::SQL_HANDLE_STMT,
                    *ptr::addr_of!(stmt_1).cast::<Handle>()
                ),
            );

            // Unbind statement 2
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_2, FreeStmtOption::SQL_UNBIND as i16),
                "{}",
                get_sql_diagnostics(
                    HandleType::SQL_HANDLE_STMT,
                    *ptr::addr_of!(stmt_2).cast::<Handle>()
                ),
            );

            // Free the handle for statement 3
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(
                    HandleType::SQL_HANDLE_STMT as i16,
                    *ptr::addr_of!(stmt_3).cast::<Handle>()
                ),
                "{}",
                get_sql_diagnostics(
                    HandleType::SQL_HANDLE_STMT,
                    *ptr::addr_of!(stmt_3).cast::<Handle>()
                ),
            );
        }

        disconnect_and_free_dbc_and_env_handles(env, dbc);
    }
}
