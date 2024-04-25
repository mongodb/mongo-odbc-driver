mod common;

/// This module contains tests for SQLGetData, ensuring we can handle large buffer sizes
/// for large documents.
mod integration {
    use std::{ffi::c_void, ptr};

    use crate::common::{
        default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, FreeStmtOption, HStmt, Handle, HandleType, SQLExecDirectW,
        SQLFetch, SQLFreeStmt, SQLGetData, SQLMoreResults, SQLPrepareW, SmallInt, SqlReturn,
        USmallInt, SQL_NTS,
    };
    use tailcall::tailcall;

    #[tailcall]
    unsafe fn get_data(
        statement_handle: Handle,
        col_or_param_num: usize,
        target_types: Vec<CDataType>,
        target_value_ptr: *mut c_void,
        buffer_len: isize,
        str_len_or_ind_ptr: *mut isize,
    ) {
        match SQLGetData(
            statement_handle as HStmt,
            (col_or_param_num + 1) as USmallInt,
            target_types[col_or_param_num] as i16,
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ) {
            SqlReturn::SUCCESS_WITH_INFO => get_data(
                statement_handle,
                col_or_param_num,
                target_types,
                target_value_ptr,
                buffer_len,
                str_len_or_ind_ptr,
            ),
            _ => {}
        }
    }

    pub fn fetch_and_get_data(
        stmt: Handle,
        expected_fetch_count: Option<SmallInt>,
        _expected_sql_returns: Vec<SqlReturn>,
        target_types: Vec<CDataType>,
        buffer_size: usize,
    ) {
        let mut successful_fetch_count = 0;
        let target_value_ptr =
            Box::into_raw(Box::from(vec![0u16; buffer_size]) as Box<[u16]>).cast::<c_void>();
        let buffer_length = isize::try_from(buffer_size * std::mem::size_of::<u16>())
            .expect("Buffer length is too large to convert to isize.");
        let str_len_or_ind_ptr = Box::into_raw(Box::from(0isize) as Box<isize>).cast::<isize>();
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
                        get_data(
                            stmt,
                            0,
                            target_types.clone(),
                            target_value_ptr,
                            buffer_length,
                            str_len_or_ind_ptr,
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
            let _ = Box::from_raw(target_value_ptr.cast::<Vec<u16>>());
            let _ = Box::from_raw(str_len_or_ind_ptr.cast::<isize>());
        }
    }
    #[test]
    fn get_data_with_various_buffer_sizes() {
        // We test different size buffers to ensure we can handle various buffer sizes.
        let buffer_sizes = [
            u8::MAX as usize,
            u16::MAX as usize,
            1024 * 1024 * 8,
            u32::MAX as usize,
        ];

        for buffer_size in buffer_sizes {
            let (env_handle, conn_handle, stmt_handle) =
                default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

            unsafe {
                let query = b"SELECT * FROM integration_test.class\0".map(|c| c.into());
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLPrepareW(
                        stmt_handle,
                        query.as_ptr(),
                        i32::try_from(SQL_NTS).expect("SQL_NTS is too large to convert to i32")
                    ),
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
                );

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLExecDirectW(
                        stmt_handle,
                        query.as_ptr(),
                        i32::try_from(SQL_NTS).expect("SQL_NTS is too large to convert to i32")
                    ),
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
                );

                fetch_and_get_data(
                    *ptr::addr_of!(stmt_handle).cast::<Handle>(),
                    Some(5),
                    vec![SqlReturn::SUCCESS_WITH_INFO, SqlReturn::SUCCESS],
                    vec![CDataType::SQL_C_WCHAR],
                    buffer_size,
                );

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_CLOSE as i16),
                    "{}",
                    get_sql_diagnostics(
                        HandleType::SQL_HANDLE_STMT,
                        *ptr::addr_of!(stmt_handle).cast::<Handle>()
                    )
                );

                disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
            }
        }
    }
}
