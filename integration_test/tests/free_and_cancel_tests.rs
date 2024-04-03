mod common;

/// The tests in this module are based on the Preview Data workflow using SSIS.
/// Although that tool is the inspiration for these tests, they more generally
/// test what happens when
///   1. a user allocates and uses a statement, and then calls SQLFreeStmt
///   2. a user allocates and uses a statement to execute a query, and then
///      calls SQLCancel
/// These are workflows that could appear in any ODBC use-case, not just SSIS.
mod integration {
    use crate::common::{
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        fetch_and_get_data, get_sql_diagnostics,
    };
    use cstr::WideChar;
    use definitions::{
        AttrOdbcVersion, CDataType, FreeStmtOption, Handle, HandleType, Pointer, SQLCancel,
        SQLExecDirectW, SQLFreeStmt, SQLPrepareW, SQLSetStmtAttrW, SqlReturn, StatementAttribute,
        SQL_NTS,
    };

    /// This test is inspired by the SSIS Preview Data result set metadata flow.
    /// It is altered to be more general than that specific flow, with a focus
    /// on freeing the statement handle after preparing a statement. This flow
    /// depends on setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLPrepareW(<query>)
    ///     - SQLFreeStmt(SQL_CLOSE)
    ///     - SQLDisconnect
    ///     - SQLFreeHandle(SQL_HANDLE_DBC)
    ///     - SQLFreeHandle(SQL_HANDLE_ENV)
    #[test]
    fn test_free_stmt() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            let mut query: Vec<WideChar> =
                cstr::to_widechar_vec("SELECT * FROM integration_test.foo");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt_handle, query.as_ptr(), SQL_NTS as i32),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_CLOSE as i16),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
        }
    }

    /// This test is inspired by the SSIS Preview Data data retrieval flow.
    /// It is altered to be more general than that specific flow, with a focus
    /// on canceling the query after getting some data. This flow depends on
    /// default_setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLSetStmtAttrW(SQL_ATTR_QUERY_TIMEOUT, 15)
    ///     - SQLExecDirectW(<query>)
    ///     - <loop: until SQLFetch return SQL_NO_DATA>
    ///         - SQLFetch
    ///         - SQLGetData
    ///     - SQLCancel
    #[test]
    fn test_cancel_noop() {
        let (_, _, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            let timeout: i32 = 15;
            let value = timeout as Pointer;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetStmtAttrW(
                    stmt_handle,
                    StatementAttribute::SQL_ATTR_QUERY_TIMEOUT as i32,
                    value,
                    0
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            let mut query: Vec<WideChar> =
                cstr::to_widechar_vec("SELECT * FROM integration_test.foo");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt_handle, query.as_ptr(), SQL_NTS as i32),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            fetch_and_get_data(
                stmt_handle as Handle,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_SBIGINT],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLCancel(stmt_handle),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            )
        }
    }
}
