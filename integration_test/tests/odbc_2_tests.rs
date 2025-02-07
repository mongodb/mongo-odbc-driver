#![allow(
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

mod common;

mod integration {
    use crate::common::{
        allocate_env, connect_and_allocate_statement, default_setup_connect_and_alloc_stmt,
        disconnect_and_close_handles, generate_default_connection_str, get_sql_diagnostics,
        BUFFER_LENGTH,
    };
    use definitions::{
        AttrOdbcVersion, CDataType, HStmt, Handle, HandleType, Len, Pointer, SQLFetch, SQLGetData,
        SQLGetTypeInfo, SQLTablesW, SmallInt, SqlDataType, SqlReturn,
    };

    use cstr::WideChar;

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

            let no_data = WideChar::default();

            // list tables with null pointers for the table strings
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLTablesW(
                    stmt_handle as HStmt,
                    std::ptr::addr_of!(no_data),
                    0,
                    std::ptr::addr_of!(no_data),
                    0,
                    std::ptr::addr_of!(no_data),
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

    const EXPECTED_DATATYPES: [SqlDataType; 33] = [
        SqlDataType::SQL_WLONGVARCHAR,   // -10
        SqlDataType::SQL_WVARCHAR,       // -9
        SqlDataType::SQL_WCHAR,          // -8
        SqlDataType::SQL_BIT,            // -7
        SqlDataType::SQL_TINYINT,        // -6
        SqlDataType::SQL_BIGINT,         // -5
        SqlDataType::SQL_LONGVARBINARY,  // -4
        SqlDataType::SQL_VARBINARY,      // -3
        SqlDataType::SQL_BINARY,         // -2
        SqlDataType::SQL_LONGVARCHAR,    // -1
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_UNKNOWN_TYPE,   //0
        SqlDataType::SQL_CHAR,           //1
        SqlDataType::SQL_INTEGER,        //4
        SqlDataType::SQL_SMALLINT,       //5
        SqlDataType::SQL_FLOAT,          //6
        SqlDataType::SQL_REAL,           //7
        SqlDataType::SQL_DOUBLE,         //8
        SqlDataType::SQL_TIMESTAMP,      //11
        SqlDataType::SQL_VARCHAR,        //12
        SqlDataType::SQL_TYPE_TIMESTAMP, //93
    ];

    /// call SQLGetTypeInfo to verify the correct types are returned. For all types,
    /// we should get both date types back. For date, we should get the specific date type
    /// we expect back.
    #[test]
    fn test_type_listing() {
        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC2);
        let mut conn_str = generate_default_connection_str();
        conn_str.push_str("SIMPLE_TYPES_ONLY=0;");
        let (conn_handle, stmt_handle) = connect_and_allocate_statement(env_handle, Some(conn_str));

        let output_buffer = Box::into_raw(Box::new([0u16; BUFFER_LENGTH as usize - 1]));
        let mut text_len_or_ind = 0;

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
                        CDataType::SQL_C_LONG as i16,
                        output_buffer as Pointer,
                        (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                        std::ptr::addr_of_mut!(text_len_or_ind)
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
                    CDataType::SQL_C_LONG as i16,
                    output_buffer as Pointer,
                    (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                    std::ptr::addr_of_mut!(text_len_or_ind)
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
