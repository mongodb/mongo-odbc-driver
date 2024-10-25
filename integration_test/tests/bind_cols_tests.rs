#![allow(
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
mod common;

mod integration {
    use crate::common::{
        bind_cols, default_setup_connect_and_alloc_stmt, exec_direct_default_query,
        get_sql_diagnostics,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, FetchOrientation, FreeStmtOption, Handle, HandleType, Integer,
        Len, Pointer, SQLFetchScroll, SQLFreeStmt, SQLSetStmtAttrW, SmallInt, SqlReturn,
        StatementAttribute, ULen,
    };

    // The `_id` field is an int that is used across multiple tests.
    const ID_TRANSFER_OCTET_LEN: usize = 4;

    // The `a` field is a long that is used across multiple tests.
    const A_TRANSFER_OCTET_LEN: usize = 8;

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
        let (_, _, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3, None);

        unsafe {
            exec_direct_default_query(stmt_handle);

            let id_buffer = &mut [0u8; ID_TRANSFER_OCTET_LEN];
            let id_indicator = &mut [0isize; 1] as *mut Len;
            let a_buffer = &mut [0u8; A_TRANSFER_OCTET_LEN];
            let a_indicator = &mut [0isize; 1] as *mut Len;

            bind_cols(
                stmt_handle,
                vec![
                    (
                        CDataType::SQL_C_SLONG,
                        id_buffer as *mut u8 as Pointer,
                        ID_TRANSFER_OCTET_LEN as Len,
                        id_indicator,
                    ),
                    (
                        CDataType::SQL_C_SBIGINT,
                        a_buffer as *mut u8 as Pointer,
                        A_TRANSFER_OCTET_LEN as Len,
                        a_indicator,
                    ),
                ],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFetchScroll(stmt_handle, FetchOrientation::SQL_FETCH_NEXT as SmallInt, 0,),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt_handle, FreeStmtOption::SQL_UNBIND as SmallInt),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );
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
        let (_, _, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC3, None);

        unsafe {
            const ROW_ARRAY_SIZE: usize = 2;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetStmtAttrW(
                    stmt_handle,
                    StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE as Integer,
                    ROW_ARRAY_SIZE as Pointer,
                    0
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
            );

            exec_direct_default_query(stmt_handle);

            // _id is an int
            const ID_BUFFER_LEN: usize = ROW_ARRAY_SIZE * ID_TRANSFER_OCTET_LEN;
            let id_buffer = &mut [0u8; ID_BUFFER_LEN] as *mut _;
            let id_indicator = &mut [0isize; ROW_ARRAY_SIZE];

            // a is a long
            const A_TRANSFER_OCTET_LEN: usize = 8;
            const A_BUFFER_LEN: usize = ROW_ARRAY_SIZE * A_TRANSFER_OCTET_LEN;
            let a_buffer = &mut [0u8; A_BUFFER_LEN] as *mut _;
            let a_indicator = &mut [0isize; ROW_ARRAY_SIZE];

            bind_cols(
                stmt_handle,
                vec![
                    (
                        CDataType::SQL_C_SLONG,
                        id_buffer as Pointer,
                        ID_TRANSFER_OCTET_LEN as Len,
                        id_indicator as *mut Len,
                    ),
                    (
                        CDataType::SQL_C_SBIGINT,
                        a_buffer as Pointer,
                        A_TRANSFER_OCTET_LEN as Len,
                        a_indicator as *mut Len,
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
                assert_eq!(
                    SqlReturn::SUCCESS,
                    result,
                    "{}",
                    get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt_handle as Handle)
                );

                // After fetching the data, check that the buffers contain the
                // expected values. As shown above, the data only has 3 rows.
                assert!(i < 2, "too much data fetched");

                let (id1, id2) = expected_id_data[i];
                assert_eq!(id1, *(id_buffer as *mut i32));
                assert_eq!(
                    id2,
                    *((id_buffer as ULen + ID_TRANSFER_OCTET_LEN) as *mut i32)
                );
                assert_eq!(ID_TRANSFER_OCTET_LEN as isize, id_indicator[0]);
                assert_eq!(ID_TRANSFER_OCTET_LEN as isize, id_indicator[1]);

                let (a1, a2) = expected_a_data[i];
                assert_eq!(a1, *(a_buffer as *mut i64));
                assert_eq!(a2, *((a_buffer as ULen + A_TRANSFER_OCTET_LEN) as *mut i64));
                assert_eq!(A_TRANSFER_OCTET_LEN as isize, a_indicator[0]);
                assert_eq!(A_TRANSFER_OCTET_LEN as isize, a_indicator[1]);

                i += 1;
            }
        }
    }
}
