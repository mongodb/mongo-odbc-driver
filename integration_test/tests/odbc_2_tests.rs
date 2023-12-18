mod common;

mod integration {
    use crate::common::{
        connect_and_allocate_statement, connect_with_conn_string, fetch_and_get_data,
        get_sql_diagnostics, BUFFER_LENGTH,
    };
    use odbc_sys::{
        CDataType, EnvironmentAttribute, HEnv, HStmt, Handle, HandleType, Len, Pointer,
        SQLAllocHandle, SQLExecDirectW, SQLFetch, SQLGetData, SQLGetTypeInfo, SQLSetEnvAttr,
        SQLTablesW, SmallInt, SqlDataType, SqlReturn, NTS,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;

    // set up env handle and set odbc version to 2
    fn setup() -> odbc_sys::HEnv {
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
                    EnvironmentAttribute::OdbcVersion,
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
        let conn_str = crate::common::generate_default_connection_str();
        let conn_handle = connect_with_conn_string(env_handle, conn_str).unwrap();
        let mut stmt: Handle = null_mut();

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(
                    HandleType::Stmt,
                    conn_handle as *mut _,
                    &mut stmt as *mut Handle
                )
            );
            let mut table_view: Vec<WideChar> = cstr::to_widechar_vec("TABLE");
            table_view.push(0);

            // list tables with null pointers for the table strings
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLTablesW(
                    stmt as HStmt,
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
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as HStmt));
            }

            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt as HStmt))
        }
    }

    const EXPECTED_DATATYPES: [SqlDataType; 22] = [
        SqlDataType::EXT_W_VARCHAR,
        SqlDataType::EXT_BIT,
        SqlDataType::EXT_BIG_INT,
        SqlDataType::EXT_BINARY,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::UNKNOWN_TYPE,
        SqlDataType::INTEGER,
        SqlDataType::DOUBLE,
        SqlDataType::EXT_TIMESTAMP,
        SqlDataType::TIMESTAMP,
    ];

    /// call SQLGetTypeInfo to verify the correct types are returned. For all types,
    /// we should get both date types back. For date, we should get the specific date type
    /// we expect back.
    #[test]
    fn test_type_listing() {
        let env_handle = setup();
        let (_, stmt_handle) = connect_and_allocate_statement(env_handle, None);

        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

        unsafe {
            // check that when requesting all types, both odbc 2 and 3 timestamp types are returned
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::ALL_TYPES)
            );
            for datatype in EXPECTED_DATATYPES {
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as HStmt,
                        2,
                        CDataType::SLong,
                        output_buffer as Pointer,
                        (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                        null_mut()
                    )
                );
                assert_eq!(*(output_buffer as *mut i16), datatype.0);
            }

            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as HStmt));

            // test that SQLGetTypeInfo works properly with odbc 2.x date type
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::EXT_TIMESTAMP)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetData(
                    stmt_handle as HStmt,
                    2,
                    CDataType::SLong,
                    output_buffer as Pointer,
                    (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                    null_mut()
                )
            );
            assert_eq!(*(output_buffer as *mut i16), 11);

            // test that SQLGetTypeInfo returns an error for odbc 3.x date type
            assert_eq!(
                SqlReturn::ERROR,
                SQLGetTypeInfo(stmt_handle as HStmt, SqlDataType::TIMESTAMP)
            );
        }
    }

    #[test]
    fn test_data_retrieval() {
        let env_handle = setup();
        let (_, stmt_handle) = connect_and_allocate_statement(env_handle, None);

        unsafe {
            // query for date, which is stored as a string. We'll then convert to date using the odbc 2 type
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select `stardate` from class");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt_handle as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt_handle as Handle)
            );

            // fetch date field as odbc 2.x TimeStamp
            fetch_and_get_data(
                stmt_handle as *mut _,
                Some(5),
                vec![SqlReturn::SUCCESS; 1],
                vec![CDataType::TimeStamp],
            );
        }
    }
}
