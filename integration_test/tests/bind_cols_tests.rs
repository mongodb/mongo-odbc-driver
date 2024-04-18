mod common;

mod integration {
    use crate::common::{
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        exec_direct_default_query, fetch_and_bind_cols, get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, BindType, CDataType, FreeStmtOption, Handle, HandleType, Pointer,
        SQLFreeStmt, SQLSetStmtAttrW, SqlReturn, StatementAttribute,
    };

    /// This test is inspired by the SSIS Query flow. It is altered to be
    /// more general than that specific flow, with a focus on freeing the
    /// statement handle after calling SQLBindCol. This flow depends on
    /// default_setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLSetStmtAttrW(SQL_ATTR_ROW_BIND_TYPE, SQL_BIND_BY_COLUMN)
    ///     - SQLSetStmtAttrW(SQL_ATTR_ROW_ARRAY_SIZE, 1000)
    ///     - SQLExecDirectW(<query>)
    ///     - <loop: until SQLFetchScroll returns SQL_NO_DATA>
    ///         - SQLBindCol for each column
    ///         - SQLFetchScroll
    ///     - SQLFreeStmt(SQL_UNBIND)
    ///     - SQLDisconnect
    ///     - SQLFreeHandle(SQL_HANDLE_DBC)
    ///     - SQLFreeHandle(SQL_HANDLE_ENV)
    #[test]
    fn test_free_stmt_after_bind_col() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            let bind_type = BindType::SQL_BIND_BY_COLUMN as i32;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetStmtAttrW(
                    stmt_handle,
                    StatementAttribute::SQL_ATTR_ROW_BIND_TYPE,
                    bind_type as Pointer,
                    0,
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            let row_array_size = 1000;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetStmtAttrW(
                    stmt_handle,
                    StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE,
                    row_array_size as Pointer,
                    0
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            exec_direct_default_query(stmt_handle);

            fetch_and_bind_cols(
                stmt_handle,
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_SBIGINT],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_UNBIND),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
        }
    }
}
