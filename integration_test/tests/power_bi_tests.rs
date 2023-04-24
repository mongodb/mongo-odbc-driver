mod common;

mod integration {
    use crate::common::{
        generate_default_connection_str, get_sql_diagnostics, sql_return_to_string,
    };
    use odbc_sys::{
        AttrConnectionPooling, AttrOdbcVersion, CDataType, Desc, DriverConnectOption,
        EnvironmentAttribute, HDbc, HEnv, HStmt, Handle, HandleType, InfoType, Len, Pointer,
        SQLAllocHandle, SQLColAttributeW, SQLDriverConnectW, SQLExecDirectW, SQLFetch,
        SQLFreeHandle, SQLGetData, SQLGetInfoW, SQLMoreResults, SQLNumResultCols, SQLSetEnvAttr,
        SQLTablesW, SmallInt, SqlReturn, USmallInt, NTS,
    };
    #[cfg(not(target_os = "macos"))]
    use odbc_sys::{ConnectionAttribute, SQLSetConnectAttrW};

    use cstr::WideChar;
    use std::ptr::null_mut;
    use std::slice;

    const BUFFER_LENGTH: SmallInt = 300;

    macro_rules! test_get_info {
        ($conn_handle:expr, $info_type: expr, $info_value_buffer_length: expr, $info_value_type: expr) => {{
            let conn_handle = $conn_handle;
            let info_type = $info_type;
            let info_value_buffer_length = $info_value_buffer_length;
            let info_value_type = $info_value_type;

            let output_buffer = &mut [0; (BUFFER_LENGTH as usize - 1)] as *mut _;
            let mut buffer = OutputBuffer {
                output_buffer: output_buffer as Pointer,
                data_length: *&mut 0,
            };

            let outcome = SQLGetInfoW(
                conn_handle as HDbc,
                info_type,
                buffer.output_buffer,
                info_value_buffer_length,
                &mut buffer.data_length as &mut _,
            );
            assert_eq!(
                SqlReturn::SUCCESS,
                outcome,
                "Expected {}, got {}. Diagnostic message is: {}",
                sql_return_to_string(SqlReturn::SUCCESS),
                sql_return_to_string(outcome),
                get_sql_diagnostics(HandleType::Dbc, conn_handle as Handle)
            );

            let length = buffer.data_length.clone();
            println!(
                "{info_type:?} = {}\nLength is {length}",
                match info_value_type {
                    DataType::WChar => Into::<String>::into(buffer),
                    DataType::USmallInt => Into::<u16>::into(buffer).to_string(),
                }
            );
        }};
    }

    pub enum DataType {
        USmallInt,
        WChar,
    }

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

    ///  Helper function for the following Power BI flow
    ///  - SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
    ///  - SQLGetInfoW(SQL_GETDATA_EXTENSIONS)
    ///  - SQLNumResultCols()
    ///  - For columns 1 to {numCols}
    ///      - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
    ///      - SQLColAttributeW(SQL_DESC_UNSIGNED)
    ///      - SQLColAttributeW(SQL_COLUMN_NAME)
    ///      - SQLColAttributeW(SQL_COLUMN_NULLABLE)
    ///      - SQLColAttributeW(SQL_DESC_TYPE_NAME)
    ///      - SQLColAttributeW(SQL_COLUMN_LENGTH)
    ///      - SQLColAttributeW(SQL_COLUMN_SCALE)
    fn get_column_attributes(conn_handle: HDbc, stmt: Handle, expected_col_count: SmallInt) {
        // SQLGetFunctions is not available through odbc_sys
        /*
        SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
        */
        let str_len_ptr = &mut 0;
        let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;
        unsafe {
            test_get_info!(
                conn_handle,
                InfoType::GetDataExtensions,
                2,
                DataType::USmallInt
            );

            let column_count_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLNumResultCols(stmt as HStmt, column_count_ptr),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
            assert_eq!(expected_col_count, *column_count_ptr);

            let numeric_attribute_ptr = &mut 0;
            const FIELD_IDS: [Desc; 7] = [
                Desc::ConciseType,
                Desc::Unsigned,
                Desc::Name,
                Desc::Nullable,
                Desc::TypeName,
                Desc::Length,
                Desc::Scale,
            ];
            for col_num in 0..*column_count_ptr {
                FIELD_IDS.iter().for_each(|field_type| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLColAttributeW(
                            stmt as HStmt,
                            (col_num + 1) as u16,
                            *field_type,
                            output_buffer as Pointer,
                            BUFFER_LENGTH,
                            str_len_ptr,
                            numeric_attribute_ptr,
                        ),
                        "{}",
                        get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                    );
                });
            }
        }
    }

    ///  Helper function for fetching and getting data
    ///  - Until SQLFetch returns SQL_NO_DATA
    ///      - SQLFetch()
    ///      - For columns 1 to {numCols}
    ///          - SQLGetData({colIndex}, {defaultCtoSqlType})
    ///  - SQLMoreResults()
    ///  -SQLFreeHandle(SQL_HANDLE_STMT)
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
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Stmt, stmt as Handle),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
        }
    }

    /// Test PowerBI Setup flow
    #[test]
    fn test_setup() {
        setup();
    }

    /// Test PowerBi environment clean-up
    #[test]
    fn test_env_cleanup() {
        // We need a handle to be able to test that freeing the handle work
        let env_handle: HEnv = setup();

        unsafe {
            // Verify that freeing the handle is working as expected
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Env, env_handle as Handle),
                "{}",
                get_sql_diagnostics(HandleType::Env, env_handle as Handle)
            );
        }
    }

    /// Test PowerBi connection flow (the setup flow is a pre-requisite)
    /// Connection flow is :
    /// - SQLAllocHandle(SQL_HANDLE_DBC)
    /// - SQLSetConnectAttrW(SQL_ATTR_LOGIN_TIMEOUT)
    /// - SQLDriverConnectW({NullTerminatedInConnectionString}, NTS , {NullTerminatedOutConnectionString}, NTS , SQL_DRIVER_NOPROMPT)
    /// - SQLGetInfoW(SQL_DRIVER_NAME)
    /// - SQLGetInfoW(SQL_DBMS_NAME)
    /// - SQLGetInfoW(SQL_DBMS_VER)
    #[test]
    fn test_connection() {
        let env_handle: HEnv = setup();
        let (conn_handle, in_connection_string, out_connection_string, output_len) =
            power_bi_connect(env_handle);

        unsafe {
            let input_len = in_connection_string.len() as SmallInt;

            println!("Input connection string = {in_connection_string}\nLength is {input_len}");
            println!("Output connection string = {out_connection_string}\nLength is {output_len}");
            // The output string should be the same as the input string except with extra curly braces around the driver name
            assert_eq!(input_len, output_len, "Expect that both connection the input connection string and output connection string have the same length but input string length is {input_len} and output string length is {output_len}");

            // SQL_DRIVER_NAME is not accessible through odbc_sys
            /*
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetInfoW(
                    dbc as HDbc,
                    SQL_DRIVER_NAME, // 6
                    driver_name as Pointer,
                    BUFFER_LENGTH,
                    str_len_ptr
                )
            );
             */

            test_get_info!(
                conn_handle,
                InfoType::DbmsName,
                14 * (std::mem::size_of::<WideChar>() as i16),
                DataType::WChar
            );

            test_get_info!(
                conn_handle,
                InfoType::DbmsVer,
                29 * (std::mem::size_of::<WideChar>() as i16),
                DataType::WChar
            );
        }
    }

    // Test PowerBI driver information retrieval
    // This test is limited by the available InfoType values in odbc_sys
    #[test]
    fn test_get_driver_info() {
        let env_handle: HEnv = setup();
        let (conn_handle, _, _, _) = power_bi_connect(env_handle);

        unsafe {
            test_get_info!(
                conn_handle,
                InfoType::IdentifierQuoteChar,
                (2 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // SQL-1177: Investigate how to test missing InfoType values
            // InfoType::SQL_OWNER_USAGE
            // InfoType::SQL_CATALOG_USAGE
            // InfoType::SQL_CATALOG_NAME_SEPARATOR
            // InfoType::SQL_CATALOG_LOCATION
            // InfoType::SQL_SQL_CONFORMANCE
            test_get_info!(
                conn_handle,
                InfoType::MaxColumnsInOrderBy,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::MaxIdentifierLen,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::MaxColumnsInGroupBy,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::MaxColumnsInSelect,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::OrderByColumnsInSelect,
                (2 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // InfoType::SQL_STRING_FUNCTIONS
            // InfoType::SQL_AGGREGATE_FUNCTIONS
            // InfoType::SQL_SQL92_PREDICATES
            // InfoType::SQL_SQL92_RELATIONAL_JOIN_OPERATORS
            // InfoType::SQL_COLUMN_ALIAS
            // InfoType::SQL_GROUP_BY
            // InfoType::SQL_NUMERIC_FUNCTIONS
            // InfoType::SQL_TIMEDATE_FUNCTIONS
            // InfoType::SQL_SYSTEM_FUNCTIONS
            // InfoType::SQL_TIMEDATE_ADD_INTERVALS
            // InfoType::SQL_TIMEDATE_DIFF_INTERVALS
            // InfoType::SQL_CONCAT_NULL_BEHAVIOR
            test_get_info!(
                conn_handle,
                InfoType::CatalogName,
                (2 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // InfoType::SQL_CATALOG_TERM
            // InfoType::SQL_OWNER_TERM
            // InfoType::SQL_ODBC_INTERFACE_CONFORMANCE
            test_get_info!(
                conn_handle,
                InfoType::SearchPatternEscape,
                (2 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // InfoType::SQL_CONVERT_FUNCTIONS
            // InfoType::SQL_CONVERT_BIGINT
            // InfoType::SQL_CONVERT_BINARY
            // InfoType::SQL_CONVERT_BIT
            // InfoType::SQL_CONVERT_CHAR
            // InfoType::SQL_CONVERT_DECIMAL
            // InfoType::SQL_CONVERT_DOUBLE
            // InfoType::SQL_CONVERT_FLOAT
            // InfoType::SQL_CONVERT_GUID
            // InfoType::SQL_CONVERT_INTEGER
            // InfoType::SQL_CONVERT_LONGVARBINARY
            // InfoType::SQL_CONVERT_LONGVARCHAR
            // InfoType::SQL_CONVERT_NUMERIC
            // InfoType::SQL_CONVERT_REAL
            // InfoType::SQL_CONVERT_SMALLINT
            // InfoType::SQL_CONVERT_TIMESTAMP
            // InfoType::SQL_CONVERT_TINYINT
            // InfoType::SQL_CONVERT_DATE
            // InfoType::SQL_CONVERT_TIME
            // InfoType::SQL_CONVERT_VARBINARY
            // InfoType::SQL_CONVERT_VARCHAR
            // InfoType::SQL_CONVERT_WCHAR
            // InfoType::SQL_CONVERT_WLONGVARCHAR
            // InfoType::SQL_CONVERT_WVARCHAR
            test_get_info!(
                conn_handle,
                InfoType::SpecialCharacters,
                (22 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // InfoType::SQL_RETURN_ESCAPE_CLAUSE
            // InfoType::SQL_DRIVER_ODBC_VER
        }
    }

    /// Test PowerBi data retrieval flow (setup and connection flows are prerequisites)
    /// data retrieval flow is:
    ///     - SQLAllocHandle(SQL_HANDLE_STMT)
    ///     - SQLGetInfoW(SQL_DRIVER_ODBC_VER)
    ///     - SQLGetInfoW(SQL_DRIVER_NAME)
    ///     - SQLExecDirectW({NullTerminatedQuery},NTS)
    ///     - SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
    ///     - SQLGetInfoW(SQL_GETDATA_EXTENSIONS)
    ///     - SQLNumResultCols()
    ///     - For columns 1 to {numCols}
    ///         - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
    ///         - SQLColAttributeW(SQL_DESC_UNSIGNED)
    ///         - SQLColAttributeW(SQL_COLUMN_NAME)
    ///         - SQLColAttributeW(SQL_COLUMN_NULLABLE)
    ///         - SQLColAttributeW(SQL_DESC_TYPE_NAME)
    ///         - SQLColAttributeW(SQL_COLUMN_LENGTH)
    ///         - SQLColAttributeW(SQL_COLUMN_SCALE)
    ///     - For X rows or until SQLFetch returns SQL_NO_DATA
    ///         - SQLFetch()
    ///         - For columns 1 to {numCols}
    ///             - SQLGetData({colIndex}, {defaultCtoSqlType})
    ///     - SQLMoreResults()
    ///     - SQLFreeHandle(SQL_HANDLE_STMT)
    #[test]
    fn test_data_retrieval() {
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

            // SQL_DRIVER_ODBC_VER and SQL_DRIVER_NAME are not available through odbc_sys
            /*
            SQLGetInfoW(SQL_DRIVER_ODBC_VER)
            SQLGetInfoW(SQL_DRIVER_NAME)
            */
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            get_column_attributes(conn_handle, stmt, 2);
            fetch_and_get_data(
                stmt,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SLong, CDataType::WChar],
            );
        }
    }

    ///  Test PowerBI flow for listing tables
    ///  - SQLAllocHandle(SQL_HANDLE_STMT)
    ///  - SQLTablesW(null pointer, null pointer, null pointer, "TABLE,VIEW")
    ///  - SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
    ///  - SQLGetInfoW(SQL_GETDATA_EXTENSIONS)
    ///  - SQLNumResultCols()
    ///  - For columns 1 to {numCols}
    ///      - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
    ///      - SQLColAttributeW(SQL_DESC_UNSIGNED)
    ///      - SQLColAttributeW(SQL_COLUMN_NAME)
    ///      - SQLColAttributeW(SQL_COLUMN_NULLABLE)
    ///      - SQLColAttributeW(SQL_DESC_TYPE_NAME)
    ///      - SQLColAttributeW(SQL_COLUMN_LENGTH)
    ///      - SQLColAttributeW(SQL_COLUMN_SCALE)
    ///  - Until SQLFetch returns SQL_NO_DATA
    ///      - SQLFetch()
    ///      - For columns 1 to {numCols}
    ///          - SQLGetData({colIndex}, {defaultCtoSqlType})
    ///  - SQLMoreResults()
    ///  -SQLFreeHandle(SQL_HANDLE_STMT)
    #[test]
    fn test_table_listing() {
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
            let mut table_view: Vec<WideChar> = cstr::to_widechar_vec("TABLE,VIEW");
            table_view.push(0);
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

            get_column_attributes(conn_handle, stmt, 5);
            fetch_and_get_data(
                stmt,
                None,
                vec![
                    SqlReturn::SUCCESS,
                    SqlReturn::SUCCESS,
                    SqlReturn::SUCCESS,
                    SqlReturn::SUCCESS,
                    SqlReturn::NO_DATA,
                ],
                vec![CDataType::WChar; 5],
            );
        }
    }
}
