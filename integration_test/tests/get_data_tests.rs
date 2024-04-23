mod common;

/// The tests in this module are based on the Preview Data workflow using SSIS.
/// Although that tool is the inspiration for these tests, they more generally
/// test what happens when
///   1. a user allocates and uses a statement, and then calls SQLFreeStmt
///   2. a user allocates and uses a statement to execute a query, and then
///      calls SQLCancel
/// These are workflows that could appear in any ODBC use-case, not just SSIS.
mod integration {
    use std::ffi::c_void;

    use crate::common::{
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        get_sql_diagnostics,
    };
    use cstr::WideChar;
    use definitions::{
        AttrOdbcVersion, CDataType, FreeStmtOption, HStmt, Handle, HandleType, Len, Pointer,
        SQLCancel, SQLExecDirectW, SQLFetch, SQLFreeStmt, SQLGetData, SQLMoreResults, SQLPrepareW,
        SQLSetStmtAttrW, SmallInt, SqlReturn, StatementAttribute, USmallInt, SQL_NTS,
    };
    pub fn fetch_and_get_data(
        stmt: Handle,
        expected_fetch_count: Option<SmallInt>,
        expected_sql_returns: Vec<SqlReturn>,
        target_types: Vec<CDataType>,
        buffer_length: usize,
    ) {
        let mut output_buffer = vec![0u16; buffer_length];
        let mut successful_fetch_count = 0;
        let str_len_ptr = &mut 0;
        unsafe {
            loop {
                let result = SQLFetch(stmt as HStmt);
                assert!(
                    result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA,
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
                );
                match result {
                    SqlReturn::SUCCESS | SqlReturn::SUCCESS_WITH_INFO => {
                        successful_fetch_count += 1;
                        for col_num in 0..target_types.len() {
                            assert_eq!(
                                expected_sql_returns[col_num],
                                SQLGetData(
                                    stmt as HStmt,
                                    (col_num + 1) as USmallInt,
                                    target_types[col_num] as i16,
                                    output_buffer.as_mut_ptr().cast::<c_void>(),
                                    (output_buffer.len() as i16 * std::mem::size_of::<u16>() as i16)
                                        as Len,
                                    str_len_ptr
                                ),
                                "{}",
                                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
                            );
                        }
                    }
                    // break if SQLFetch returns SQL_NO_DATA
                    _ => break,
                }
            }

            if let Some(exp_fetch_count) = expected_fetch_count {
                assert_eq!(
                exp_fetch_count, successful_fetch_count,
                "Expected {exp_fetch_count:?} successful calls to SQLFetch, got {successful_fetch_count}."
            );
            }

            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));
        }
    }
    #[test]
    fn get_big_data() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            let query = b"SELECT bids FROM integration_test.bidsDebug\0".map(|c| c as u16);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt_handle, query.as_ptr(), SQL_NTS as i32),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt_handle, query.as_ptr(), SQL_NTS as i32),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            fetch_and_get_data(
                stmt_handle as Handle,
                Some(195),
                vec![SqlReturn::SUCCESS_WITH_INFO, SqlReturn::SUCCESS],
                vec![CDataType::SQL_C_WCHAR],
                2047,
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
}
