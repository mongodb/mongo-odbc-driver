mod common;

mod integration {
    use crate::common::{
        allocate_env, default_setup_connect_and_alloc_stmt, disconnect_and_close_handles,
        get_sql_diagnostics, BUFFER_LENGTH,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, HStmt, Handle, HandleType, Len, Pointer, SQLFetch, SQLGetData,
        SQLGetTypeInfo, SQLTablesW, SmallInt, SqlDataType, SqlReturn,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;

    /// Test Setup flow
    #[test]
    fn test_setup() {
        allocate_env(AttrOdbcVersion::SQL_OV_ODBC2);
    }

    /// Test list_tables, which should yield all tables
    #[test]
    fn test_list_tables() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC2);

        unsafe {
            let mut table_view: Vec<WideChar> = cstr::to_widechar_vec("TABLE");
            table_view.push(0);

            // list tables with null pointers for the table strings
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLTablesW(
                    stmt_handle as HStmt,
                    null_mut(),
                    0,
                    null_mut(),
                    0,
                    null_mut(),
                    0,
                    table_view.as_ptr(),
                    table_view.len() as SmallInt - 1
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env_handle as Handle)
            );

            // assert all tables are returned from the previous SQLTables call
            for _ in 0..14 {
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
            }

            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as HStmt));
            disconnect_and_close_handles(conn_handle, stmt_handle);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }

    const EXPECTED_DATATYPES: [SqlDataType; 23] = [
        SqlDataType::SQL_WVARCHAR,
        SqlDataType::SQL_BIT,
        SqlDataType::SQL_BIGINT,
        SqlDataType::SQL_BINARY,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_UNKNOWN_TYPE,
        SqlDataType::SQL_INTEGER,
        SqlDataType::SQL_DOUBLE,
        SqlDataType::SQL_TIMESTAMP,
        SqlDataType::SQL_VARCHAR,
        SqlDataType::SQL_TYPE_TIMESTAMP,
    ];

    /// call SQLGetTypeInfo to verify the correct types are returned. For all types,
    /// we should get both date types back. For date, we should get the specific date type
    /// we expect back.
    #[test]
    fn test_type_listing() {
        let (env_handle, conn_handle, stmt_handle) =
            default_setup_connect_and_alloc_stmt(AttrOdbcVersion::SQL_OV_ODBC2);

        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

        unsafe {
            // check that when requesting all types, both odbc 2 and 3 timestamp types are returned
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_UNKNOWN_TYPE as i16)
            );
            for datatype in EXPECTED_DATATYPES {
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as HStmt,
                        2,
                        CDataType::SQL_C_SLONG as i16,
                        output_buffer as Pointer,
                        (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                        null_mut()
                    )
                );
                assert_eq!(*(output_buffer as *mut i16), datatype as i16);
            }

            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as HStmt));

            // test that SQLGetTypeInfo works properly with odbc 2.x date type
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_TIMESTAMP as i16)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetData(
                    stmt_handle as HStmt,
                    2,
                    CDataType::SQL_C_SLONG as i16,
                    output_buffer as Pointer,
                    (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                    null_mut()
                )
            );
            assert_eq!(*(output_buffer as *mut i16), 11);

            // test that SQLGetTypeInfo returns an error for odbc 3.x date type
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_TIMESTAMP as i16)
            );
            disconnect_and_close_handles(conn_handle, stmt_handle);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }
}
