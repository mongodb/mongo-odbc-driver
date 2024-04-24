mod common;

/// This module contains tests for SQLGetData, ensuring we can handle large buffer sizes
/// for large documents.
mod integration {
    use std::ffi::c_void;

    use crate::common::{
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, FreeStmtOption, HStmt, Handle, HandleType, Len, SQLExecDirectW,
        SQLFetch, SQLFreeStmt, SQLGetData, SQLMoreResults, SQLPrepareW, SmallInt, SqlReturn,
        USmallInt, SQL_NTS,
    };
    use tailcall::tailcall;

    #[tailcall]
    unsafe fn get_data(
        stmt: Handle,
        col_num: usize,
        target_types: Vec<CDataType>,
        buffer_length: usize,
        successful_fetch_count: usize,
    ) {
        let mut output_buffer = vec![0u16; buffer_length];
        let str_len_ptr = &mut 0;
        let len = output_buffer.len() * std::mem::size_of::<u16>();
        tracing::error!(length = &len);

        match SQLGetData(
            stmt as HStmt,
            (col_num + 1) as USmallInt,
            target_types[col_num] as i16,
            output_buffer.as_mut_ptr().cast::<c_void>(),
            (output_buffer.len() * std::mem::size_of::<u16>()) as Len,
            str_len_ptr,
        ) {
            SqlReturn::SUCCESS_WITH_INFO => get_data(
                stmt,
                col_num,
                target_types,
                buffer_length,
                successful_fetch_count,
            ),
            _ => {}
        }
    }

    pub fn fetch_and_get_data(
        stmt: Handle,
        expected_fetch_count: Option<SmallInt>,
        _expected_sql_returns: Vec<SqlReturn>,
        target_types: Vec<CDataType>,
        buffer_length: usize,
    ) {
        let mut successful_fetch_count = 0;
        unsafe {
            loop {
                let result = SQLFetch(stmt as HStmt);
                assert!(
                    result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA,
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
                );
                match result {
                    SqlReturn::SUCCESS => {
                        successful_fetch_count += 1;
                        tracing::error!(fetch_count = %successful_fetch_count);
                        get_data(
                            stmt,
                            0,
                            target_types.clone(),
                            buffer_length,
                            successful_fetch_count,
                        )
                    }
                    // break if SQLFetch returns SQL_NO_DATA
                    _ => break,
                }
            }

            if let Some(exp_fetch_count) = expected_fetch_count {
                assert_eq!(
                exp_fetch_count as usize, successful_fetch_count,
                "Expected {exp_fetch_count:?} successful calls to SQLFetch, got {successful_fetch_count}."
            );
            }

            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));
        }
    }
    #[test]
    fn get_big_data_with_u16_max_plus_2_buffer() {
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
                Some(147),
                vec![SqlReturn::SUCCESS_WITH_INFO, SqlReturn::SUCCESS],
                vec![CDataType::SQL_C_WCHAR],
                u16::MAX as usize + 2,
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
    #[test]
    fn get_big_data_with_u32_max_plus_2_buffer() {
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
                Some(147),
                vec![SqlReturn::SUCCESS_WITH_INFO, SqlReturn::SUCCESS],
                vec![CDataType::SQL_C_WCHAR],
                u32::MAX as usize + 2,
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
