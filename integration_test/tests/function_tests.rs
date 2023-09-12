mod common;

mod functions {
    use crate::common::{generate_default_connection_str, get_sql_diagnostics};
    use odbc_sys::{
        AttrConnectionPooling, AttrOdbcVersion, CDataType, ConnectionAttribute,
        DriverConnectOption, EnvironmentAttribute, HDbc, HEnv, HStmt, Handle, HandleType, Len,
        Pointer, SQLAllocHandle, SQLDriverConnectW, SQLExecute, SQLFetch, SQLFreeHandle,
        SQLGetData, SQLMoreResults, SQLPrepareW, SQLSetConnectAttrW, SQLSetEnvAttr, SmallInt,
        SqlReturn, USmallInt, NTS,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;
    use std::slice;

    const BUFFER_LENGTH: SmallInt = 300;

    pub struct OutputBuffer {
        pub output_buffer: Pointer,
        pub data_length: i16,
    }

    impl From<OutputBuffer> for String {
        fn from(val: OutputBuffer) -> Self {
            unsafe {
                cstr::from_widechar_ref_lossy(slice::from_raw_parts(
                    val.output_buffer as *const _,
                    val.data_length as usize / std::mem::size_of::<WideChar>(),
                ))
            }
        }
    }

    impl From<OutputBuffer> for u16 {
        fn from(val: OutputBuffer) -> Self {
            unsafe { *(val.output_buffer as *mut u16) }
        }
    }

    /// Setup flow.
    /// This will allocate a new environment handle and set ODBC_VERSION and CONNECTION_POOLING environment attributes.
    /// Setup flow is:
    ///     - SQLAllocHandle(SQL_HANDLE_ENV)
    ///     - SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
    ///     - SQLSetEnvAttr(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
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
                    AttrOdbcVersion::Odbc3.into(),
                    0,
                )
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetEnvAttr(
                    env as HEnv,
                    EnvironmentAttribute::ConnectionPooling,
                    AttrConnectionPooling::OnePerHenv.into(),
                    0,
                )
            );
        }

        env as HEnv
    }

    /// Generate the default connection string and returns :
    /// - The connection handle
    /// - The string used as the input connection string
    /// - The retrieved output connection string
    /// - The retrieved length of the output connection string
    fn power_bi_connect(env_handle: HEnv) -> (odbc_sys::HDbc, String, String, SmallInt) {
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

            // Generate the connection string and add a null terminator because PowerBi uses NTS for the length
            in_connection_string = generate_default_connection_str();
            let mut in_connection_string_encoded = cstr::to_widechar_vec(&in_connection_string);
            in_connection_string_encoded.push(0);

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

    ///  Helper function for fetching and getting data
    ///  - Until SQLFetch returns SQL_NO_DATA
    ///      - SQLFetch()
    ///      - For columns 1 to {numCols}
    ///          - SQLGetData({colIndex}, {defaultCtoSqlType})
    ///  - SQLMoreResults()
    fn fetch_and_get_data(
        stmt: Handle,
        expected_fetch_count: Option<SmallInt>,
        expected_sql_returns: Vec<SqlReturn>,
        target_types: Vec<CDataType>,
    ) {
        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;
        let mut successful_fetch_count = 0;
        let str_len_ptr = &mut 0;
        unsafe {
            loop {
                let result = SQLFetch(stmt as HStmt);
                assert!(
                    result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA,
                    "{}",
                    get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                );
                match result {
                    SqlReturn::SUCCESS => {
                        successful_fetch_count += 1;
                        for col_num in 0..target_types.len() {
                            assert_eq!(
                                expected_sql_returns[col_num],
                                SQLGetData(
                                    stmt as HStmt,
                                    (col_num + 1) as USmallInt,
                                    target_types[col_num],
                                    output_buffer as Pointer,
                                    (BUFFER_LENGTH * std::mem::size_of::<u16>() as i16) as Len,
                                    str_len_ptr
                                ),
                                "{}",
                                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
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
    fn test_prepare_and_execute() {
        let env_handle: HEnv = setup();
        let (conn_handle, _, _, _) = power_bi_connect(env_handle);
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

            let current_db = cstr::to_widechar_ptr("integration_test");
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    conn_handle,
                    ConnectionAttribute::CurrentCatalog,
                    current_db.0 as *mut _,
                    current_db.1.len() as i32
                )
            );
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);

            assert_eq!(
                SqlReturn::ERROR,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLPrepareW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            fetch_and_get_data(
                stmt,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SLong, CDataType::WChar],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecute(stmt as HStmt),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            fetch_and_get_data(
                stmt,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SLong, CDataType::WChar],
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Stmt, stmt as Handle),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
        }
    }
}
