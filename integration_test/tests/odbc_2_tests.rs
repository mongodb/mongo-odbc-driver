mod common;

mod integration {
    use crate::common::{
        connect_and_allocate_statement, disconnect_and_close_handles, get_sql_diagnostics,
        BUFFER_LENGTH,
    };
    use definitions::{
        CDataType, EnvironmentAttribute, HEnv, HStmt, Handle, HandleType, Len, Pointer,
        SQLAllocHandle, SQLFetch, SQLGetData, SQLGetTypeInfo, SQLSetEnvAttr, SQLTablesW, SmallInt,
        SqlDataType, SqlReturn,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;

    // set up env handle and set odbc version to 2
    fn setup() -> definitions::HEnv {
        let mut env: Handle = null_mut();

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(HandleType::Env, null_mut(), &mut env as *mut Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetEnvAttr(
                    env as HEnv,
                    EnvironmentAttribute::SQL_ATTR_ODBC_VERSION,
                    2 as *mut _,
                    0,
                )
            );
        }

        env as HEnv
    }

    /// Test Setup flow
    #[test]
    fn test_setup() {
        setup();
    }

    /// Test list_tables, which should yield all tables
    #[test]
    fn test_list_tables() {
        let env_handle = setup();
        let (conn_handle, stmt_handle) = connect_and_allocate_statement(env_handle, None);

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
                get_sql_diagnostics(HandleType::Env, env_handle as Handle)
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

    const EXPECTED_DATATYPES: [SqlDataType; 22] = [
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
        SqlDataType::SQL_TYPE_TIMESTAMP,
    ];

    /// call SQLGetTypeInfo to verify the correct types are returned. For all types,
    /// we should get both date types back. For date, we should get the specific date type
    /// we expect back.
    #[test]
    fn test_type_listing() {
        let env_handle = setup();
        let (conn_handle, stmt_handle) = connect_and_allocate_statement(env_handle, None);

        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

        unsafe {
            // check that when requesting all types, both odbc 2 and 3 timestamp types are returned
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_UNKNOWN_TYPE)
            );
            for datatype in EXPECTED_DATATYPES {
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as HStmt,
                        2,
                        CDataType::SQL_C_SLONG,
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
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_TIMESTAMP)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetData(
                    stmt_handle as HStmt,
                    2,
                    CDataType::SQL_C_SLONG,
                    output_buffer as Pointer,
                    (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                    null_mut()
                )
            );
            assert_eq!(*(output_buffer as *mut i16), 11);

            // test that SQLGetTypeInfo returns an error for odbc 3.x date type
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::SQL_TIMESTAMP)
            );
            disconnect_and_close_handles(conn_handle, stmt_handle);
        }
        let _ = unsafe { Box::from_raw(env_handle) };
    }
}
