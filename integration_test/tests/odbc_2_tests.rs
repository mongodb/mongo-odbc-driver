mod common;

mod integration {
    use crate::common::{
        fetch_and_get_data, generate_default_connection_str, get_sql_diagnostics, BUFFER_LENGTH,
    };
    use odbc_sys::{
        CDataType, ConnectionAttribute, DriverConnectOption, EnvironmentAttribute, HDbc, HEnv,
        HStmt, Handle, HandleType, Len, Pointer, SQLAllocHandle, SQLDriverConnectW, SQLExecDirectW,
        SQLFetch, SQLGetData, SQLGetTypeInfo, SQLSetConnectAttrW, SQLSetEnvAttr, SQLTablesW,
        SmallInt, SqlDataType, SqlReturn, NTS,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;
    use std::slice;

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

    // create a connection handle based on the underlying env handle
    fn connect(env_handle: HEnv) -> (odbc_sys::HDbc, String, String, SmallInt) {
        // Allocate a DBC handle
        let mut dbc: Handle = null_mut();
        #[allow(unused_mut)]
        let mut output_len;
        let in_connection_string;
        let out_connection_string;
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(
                    HandleType::Dbc,
                    env_handle as *mut _,
                    &mut dbc as *mut Handle
                )
            );

            // Causes iODBC to hang on `SetConnectOptionW` call
            #[cfg(not(target_os = "macos"))]
            {
                // Set the login timeout
                let login_timeout = 15;
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLSetConnectAttrW(
                        dbc as HDbc,
                        ConnectionAttribute::LoginTimeout,
                        login_timeout as Pointer,
                        0,
                    )
                );
            }

            in_connection_string = generate_default_connection_str();
            let in_connection_string_encoded = cstr::to_widechar_vec(&in_connection_string);

            let str_len_ptr = &mut 0;
            const BUFFER_LENGTH: SmallInt = 300;
            let mut out_connection_string_buff: [WideChar; BUFFER_LENGTH as usize - 1] =
                [0; (BUFFER_LENGTH as usize - 1)];
            let out_connection_string_buff = &mut out_connection_string_buff as *mut WideChar;

            assert_ne!(
                SqlReturn::ERROR,
                SQLDriverConnectW(
                    dbc as HDbc,
                    null_mut(),
                    in_connection_string_encoded.as_ptr(),
                    NTS as SmallInt,
                    out_connection_string_buff,
                    BUFFER_LENGTH,
                    str_len_ptr,
                    DriverConnectOption::NoPrompt,
                ),
                "{}",
                get_sql_diagnostics(HandleType::Dbc, dbc)
            );

            output_len = *str_len_ptr;
            // The iodbc driver manager is multiplying the output length by size_of WideChar (u32)
            // for some reason. It is correct when returned from SQLDriverConnectW, but is 4x
            // bigger between return and here.
            if odbc_sys::USING_IODBC {
                output_len /= std::mem::size_of::<WideChar>() as i16;
            }

            out_connection_string = cstr::from_widechar_ref_lossy(slice::from_raw_parts(
                out_connection_string_buff,
                output_len as usize,
            ));
        }
        (
            dbc as HDbc,
            in_connection_string,
            out_connection_string,
            output_len,
        )
    }

    /// Test Setup flow
    #[test]
    fn test_setup() {
        setup();
    }

    /// Test connect
    #[test]
    fn test_connect() {
        let env_handle = setup();
        connect(env_handle);
    }

    /// Test list_tables, which should yield all tables
    #[test]
    fn test_list_tables() {
        let env_handle = setup();
        let (conn_handle, _, _, _) = connect(env_handle);
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
        let (conn_handle, _, _, _) = connect(env_handle);
        let mut stmt: Handle = null_mut();

        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(
                    HandleType::Stmt,
                    conn_handle as *mut _,
                    &mut stmt as *mut Handle
                )
            );

            // check that when requesting all types, both odbc 2 and 3 timestamp types are returned
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt as HStmt, SqlDataType::UNKNOWN_TYPE)
            );
            for datatype in EXPECTED_DATATYPES {
                assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as HStmt));
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt as HStmt,
                        2,
                        CDataType::SLong,
                        output_buffer as Pointer,
                        (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                        null_mut()
                    )
                );
                assert_eq!(*(output_buffer as *mut i16), datatype.0);
            }

            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt as HStmt));

            // test that SQLGetTypeInfo works properly with odbc 2.x date type
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt as HStmt, SqlDataType::EXT_TIMESTAMP)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetData(
                    stmt as HStmt,
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
                SQLGetTypeInfo(stmt as HStmt, SqlDataType::TIMESTAMP)
            );
        }
    }

    #[test]
    fn test_data_retrieval() {
        let env_handle = setup();
        connect(env_handle);
        let env_handle: HEnv = setup();
        let (conn_handle, _, _, _) = connect(env_handle);
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
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(stmt as HStmt, SqlDataType::UNKNOWN_TYPE)
            );

            // query for date, which is stored as a string. We'll then convert to date using the odbc 2 type
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select `stardate` from class");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // fetch date field as odbc 2.x TimeStamp
            fetch_and_get_data(
                stmt,
                Some(5),
                vec![SqlReturn::SUCCESS; 1],
                vec![CDataType::TimeStamp],
            );
        }
    }
}
