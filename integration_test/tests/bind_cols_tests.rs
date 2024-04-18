mod common;

mod integration {
    use crate::common::{
        bind_cols, default_setup_connect_and_alloc_stmt, disconnect_and_free_dbc_and_env_handles,
        exec_direct_default_query, get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, FetchOrientation, FreeStmtOption, Handle, HandleType, Integer,
        Len, Pointer, SQLFetchScroll, SQLFreeStmt, SQLSetStmtAttrW, SmallInt, SqlReturn,
        StatementAttribute, ULen,
    };

    /// This test is inspired by the SSIS Query flow. It is altered to be
    /// more general than that specific flow, with a focus on freeing the
    /// bound columns after calling SQLBindCol. This flow depends on
    /// default_setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLExecDirectW(<query>)
    ///     - SQLBindCol
    ///     - SQLFetchScroll
    ///     - SQLFreeStmt(SQL_UNBIND)
    #[test]
    fn test_unbind_cols() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            exec_direct_default_query(stmt_handle);

            let id_buffer = &mut [0u8; 4] as *mut _;
            let id_indicator = &mut [0isize; 2] as *mut Len;
            let a_buffer = &mut [0u8; 8] as *mut _;
            let a_indicator = &mut [0isize; 2] as *mut Len;
            bind_cols(
                stmt_handle,
                vec![
                    (
                        CDataType::SQL_C_SLONG,
                        id_buffer as Pointer,
                        4,
                        id_indicator,
                    ),
                    (
                        CDataType::SQL_C_SBIGINT,
                        a_buffer as Pointer,
                        8,
                        a_indicator,
                    ),
                ],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFetchScroll(stmt_handle, FetchOrientation::SQL_FETCH_NEXT as SmallInt, 0,)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_UNBIND as SmallInt),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            // clean up
            disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
        }
    }

    /// This test is inspired by the SSIS Query flow. It is altered to be more
    /// general than that specific flow, with a focus on binding columns and
    /// then retrieving the next rowset. This flow depends on
    /// default_setup_connect_and_alloc_stmt.
    /// After allocating a statement handle, the flow is:
    ///     - SQLSetStmtAttrW(SQL_ATTR_ROW_ARRAY_SIZE, xxx)
    ///     - SQLExecDirectW(<query>)
    ///     - SQLBindCol
    ///     - <loop: until SQLFetchScroll returns SQL_NO_DATA>
    ///         - SQLFetchScroll
    #[test]
    fn test_bind_cols_and_fetch_next_rowset() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3);

        unsafe {
            let row_array_size = 2;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetStmtAttrW(
                    stmt_handle,
                    StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE as Integer,
                    row_array_size as Pointer,
                    0
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            exec_direct_default_query(stmt_handle);

            // Since ROW_ARRAY_SIZE is set to 2, we double the buffer lengths.
            let id_buffer = &mut [0u8; 8] as *mut _;
            let id_indicator = &mut [0isize; 2] as *mut Len;
            let a_buffer = &mut [0u8; 16] as *mut _;
            let a_indicator = &mut [0isize; 2] as *mut Len;
            bind_cols(
                stmt_handle,
                vec![
                    (
                        CDataType::SQL_C_SLONG,
                        id_buffer as Pointer,
                        4,
                        id_indicator,
                    ),
                    (
                        CDataType::SQL_C_SBIGINT,
                        a_buffer as Pointer,
                        8,
                        a_indicator,
                    ),
                ],
            );

            // Data:
            // - {_id: 0, a: {$numberLong: "42"}}
            // - {_id: 1, a: {$numberLong: "13"}}
            // - {_id: 2, a: {$numberLong: "100"}}
            let mut i = 0;
            let expected_id_data = [(0i32, 1i32), (2, 1)];
            let expected_a_data = [(42i64, 13i64), (100, 13)];
            loop {
                let result =
                    SQLFetchScroll(stmt_handle, FetchOrientation::SQL_FETCH_NEXT as SmallInt, 0);
                if result == SqlReturn::NO_DATA {
                    break;
                }
                assert_eq!(SqlReturn::SUCCESS, result);

                // After fetching the data, check that the buffers contain the
                // expected values. As shown above, the data only has 3 rows.
                assert!(i < 2, "too much data fetched");

                let (id1, id2) = expected_id_data[i];
                assert_eq!(id1, *(id_buffer as *mut i32));
                assert_eq!(id2, *((id_buffer as ULen + 4) as *mut i32));
                assert_eq!(4, *id_indicator);

                let (a1, a2) = expected_a_data[i];
                assert_eq!(a1, *(a_buffer as *mut i64));
                assert_eq!(a2, *((a_buffer as ULen + 8) as *mut i64));
                assert_eq!(8, *id_indicator);

                i += 1;
            }

            // cleanup
            disconnect_and_free_dbc_and_env_handles(env_handle, conn_handle);
        }
    }
}
