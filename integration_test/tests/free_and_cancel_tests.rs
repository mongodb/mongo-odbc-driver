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
        allocate_env, connect_and_allocate_statement, disconnect_and_free_dbc_and_env_handles,
        get_column_attributes,
    };
    use cstr::WideChar;
    use definitions::{
        Desc, FreeStmtOption, HDbc, HEnv, HStmt, Handle, SQLFreeStmt, SQLPrepareW, SqlReturn,
        SQL_NTS,
    };

    /// Setup flow that connects and allocates a statement. This allocates a
    /// new environment handle, sets the ODBC_VERSION environment attribute,
    /// connects using the default URI, and allocates a statement. The flow
    /// is:
    ///     - SQLAllocHandle(SQL_HANDLE_ENV)
    ///     - SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
    ///     - SQLAllocHandle(SQL_HANDLE_DBC)
    ///     - SQLDriverConnectW
    ///     - SQLAllocHandle(SQL_HANDLE_STMT)
    fn setup_connect_and_alloc_stmt() -> (HEnv, HDbc, HStmt) {
        let env_handle = allocate_env().unwrap();
        let (conn_handle, stmt_handle) = connect_and_allocate_statement(env_handle, None);

        (env_handle, conn_handle, stmt_handle)
    }

    /// This test is inspired by the SSIS Preview Data result set metadata flow.
    /// It is altered to be more general than that specific flow, with a focus
    /// on freeing the statement handle after gathering some metadata. This flow
    /// depends on setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLPrepareW(<query>)
    ///     - SQLNumResultCols
    ///     - <loop: for each column>
    ///         - SQLDescribeColW
    ///         - SQLColAttributeW(SQL_DESC_UNSIGNED)
    ///         - SQLColAttributeW(SQL_DESC_UPDATABLE)
    ///     - SQLFreeStmt(SQL_CLOSE)
    ///     - SQLDisconnect
    ///     - SQLFreeHandle(SQL_HANDLE_DBC)
    ///     - SQLFreeHandle(SQL_HANDLE_ENV)
    #[test]
    fn test_free_stmt() {
        let (env_handle, conn_handle, stmt_handle) = setup_connect_and_alloc_stmt();

        unsafe {
            let mut query: Vec<WideChar> =
                cstr::to_widechar_vec("SELECT * FROM integration_test.foo");
            query.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt_handle, query.as_ptr(), SQL_NTS as i32),
            );

            get_column_attributes(
                stmt_handle as Handle,
                2,
                Some(vec![Desc::SQL_DESC_UNSIGNED, Desc::SQL_DESC_UPDATABLE]),
                true,
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_CLOSE)
            );

            disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
        }
    }

    #[test]
    fn test_cancel() {
        todo!()
        // todo:
        //  - implement/test "Preview Data data flow"
        //  - use setup_and_connect() as the foundation
        //  - investigate if you need to do the second set of allocations...
        //  - consider skipping some of the attr setting/getting
        //  - Preview Data data flow (with slightly alternate Connect flow)
        //      (start alt connect flow)
        //      SQLAllocHandle(SQL_HANDLE_ENV)
        //      SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION = SQL_OV_ODBC3)
        //      (next 2 aren't used later)
        //        SQLAllocHandle(SQL_HANDLE_DBC)
        //        SQLDriverConnectW
        //      (end)
        //      SQLAllocHandle(SQL_HANDLE_DBC)
        //      SQLSetConnectAttrW(SQL_ATTR_LOGIN_TIMEOUT)
        //      SQLDriverConnectW
        //      (end alt connect flow)
        //      (start Preview data data flow)
        //      SQLAllocHandle(SQL_HANDLE_STMT)
        //      SQLSetStmtAttrW(SQL_ATTR_QUERY_TIMEOUT)
        //      SQLGetInfo(SQL_DRIVER_ODBC_VER)
        //      SQLSetStmtAttrW(1228 <unknown>)
        //      SQLGetDiagFieldW
        //      SQLSetStmtAttrW(1227 <unknown>)
        //      SQLGetDiagFieldW
        //      SQLExecDirectW(<query>)
        //      SQLRowCount
        //      SQLNumResultCols
        //      <loop: for each col>
        //         SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
        //         SQLColAttributeW(SQL_DESC_UNSIGNED)
        //         SQLColAttributeW(SQL_DESC_OCTET_LENGTH)
        //         SQLColAttributeW(4 <unknown>)
        //         SQLColAttributeW(5 <unknown>)
        //         SQLColAttributeW(SQL_DESC_AUTO_UNIQUE_VALUE)
        //         SQLColAttributeW(SQL_DESC_AUTO_UPDATABLE)
        //         SQLColAttributeW(SQL_DESC_AUTO_NULLABLE)
        //     SQLColAttributeW(SQL_DESC_NAME) <col 1>
        //     SQLColAttributeW(SQL_DESC_NAME) <col 2>
        //     <loop: for each row (or perhaps for first 3 rows) (or perhaps until SQLFetch returns SQL_NO_DATA_FOUND***)>
        //        SQLFetch
        //        <loop: for each col>
        //           <if row 1>
        //              SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
        //              SQLColAttributeW(SQL_DESC_UNSIGNED)
        //           SQLGetData
        //     SQLCancel
    }
}
