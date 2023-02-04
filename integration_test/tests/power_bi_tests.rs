mod common;

mod integration {
    use crate::common::{
        generate_default_connection_str, get_sql_diagnostics, sql_return_to_string,
    };
    use odbc::ffi::SQL_NTS;
    use odbc_sys::{
        AttrConnectionPooling, AttrOdbcVersion, ConnectionAttribute, Desc, DriverConnectOption,
        EnvironmentAttribute, HDbc, HEnv, HStmt, Handle, HandleType, InfoType, Len, Pointer,
        SQLAllocHandle, SQLColAttributeW, SQLDriverConnectW, SQLExecDirectW, SQLFetch,
        SQLFreeHandle, SQLGetData, SQLGetInfoW, SQLMoreResults, SQLNumResultCols,
        SQLSetConnectAttrW, SQLSetEnvAttr, SQLTablesW, SmallInt, SqlReturn,
    };

    use std::ptr::null_mut;
    use std::slice;
    use widechar::WideChar;

    const BUFFER_LENGTH: SmallInt = 300;

    macro_rules! test_get_info {
        ($conn_handle:expr, $info_type: expr, $info_value_buffer_length: expr, $info_value_type: expr) => {{
            let conn_handle = $conn_handle;
            let info_type = $info_type;
            let info_value_buffer_length = $info_value_buffer_length;
            let info_value_type = $info_value_type;

            let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;
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
                String::from_utf16_lossy(slice::from_raw_parts(
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
        let output_len;
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

            // Generate the connection string and add a null terminator because PowerBi uses SQL_NTS for the length
            in_connection_string = generate_default_connection_str();
            let mut in_connection_string_encoded = widechar::to_widechar_vec(&in_connection_string);
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
                    SQL_NTS,
                    out_connection_string_buff,
                    BUFFER_LENGTH,
                    str_len_ptr,
                    DriverConnectOption::NoPrompt,
                ),
                "{}",
                get_sql_diagnostics(HandleType::Dbc, dbc)
            );

            output_len = *str_len_ptr;
            out_connection_string = widechar::from_widechar_ref_lossy(slice::from_raw_parts(
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
    /// - SQLDriverConnectW({NullTerminatedInConnectionString}, SQL_NTS, {NullTerminatedOutConnectionString}, SQL_NTS, SQL_DRIVER_NOPROMPT)
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

            test_get_info!(conn_handle, InfoType::DbmsName, 28, DataType::WChar);

            test_get_info!(conn_handle, InfoType::DbmsVer, 58, DataType::WChar);
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
                4,
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
                4,
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
            test_get_info!(conn_handle, InfoType::CatalogName, 4, DataType::WChar);
            // InfoType::SQL_CATALOG_TERM
            // InfoType::SQL_OWNER_TERM
            // InfoType::SQL_ODBC_INTERFACE_CONFORMANCE
            test_get_info!(
                conn_handle,
                InfoType::SearchPatternEscape,
                2,
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
                44,
                DataType::WChar
            );
            // InfoType::SQL_RETURN_ESCAPE_CLAUSE
            // InfoType::SQL_DRIVER_ODBC_VER
        }
    }

    /// Test PowerBi data retrieval flow (setup and connection flows are prerequisites)
    /// data retreival flow is:
    ///     - SQLAllocHandle(SQL_HANDLE_STMT)
    ///     - SQLGetInfoW(SQL_DRIVER_ODBC_VER)
    ///     - SQLGetInfoW(SQL_DRIVER_NAME)
    ///     - SQLExecDirectW({NullTerminatedQuery},SQL_NTS)
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

            let mut query: Vec<u16> = "select * from example".encode_utf16().collect();
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), SQL_NTS as i32),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );

            // SQLGetFunctions is not available through odbc_sys
            /*
            SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
            */

            let str_len_ptr = &mut 0;
            let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

            test_get_info!(
                conn_handle,
                InfoType::GetDataExtensions,
                2,
                DataType::USmallInt
            );

            let column_count_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLNumResultCols(stmt as HStmt, column_count_ptr)
            );
            assert_eq!(2, *column_count_ptr);

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
                            14,
                            str_len_ptr,
                            numeric_attribute_ptr,
                        ),
                        "{}",
                        get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                    );
                });
            }

            let mut successful_fetch_count = 0;
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
                        assert_eq!(
                            SqlReturn::SUCCESS,
                            SQLGetData(
                                stmt as HStmt,
                                1,
                                odbc_sys::CDataType::SLong,
                                output_buffer as Pointer,
                                2,
                                &mut 0
                            ),
                            "{}",
                            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                        );
                        assert_eq!(
                            SqlReturn::SUCCESS,
                            SQLGetData(
                                stmt as HStmt,
                                2,
                                odbc_sys::CDataType::WChar,
                                output_buffer as Pointer,
                                4,
                                &mut 0
                            ),
                            "{}",
                            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                        );
                    }
                    // break if SQLFetch returns SQL_NO_DATA
                    _ => break,
                }
            }
            assert_eq!(
                3, successful_fetch_count,
                "Expected 3 successful calls to SQLFetch, got {successful_fetch_count}."
            );

            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Stmt, stmt as Handle),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
        }
    }

    /// Test PowerBI flow for listing tables
    ///     - SQLAllocHandle(SQL_HANDLE_STMT)
    //      - SQLTablesW(null pointer, null pointer, null pointer, "TABLE,VIEW")
    //      - SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
    //      - SQLGetInfoW(SQL_GETDATA_EXTENSIONS)
    //      - SQLNumResultCols()
    //      - For columns 1 to {numCols}
    //          - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
    //          - SQLColAttributeW(SQL_DESC_UNSIGNED)
    //          - SQLColAttributeW(SQL_COLUMN_NAME)
    //          - SQLColAttributeW(SQL_COLUMN_NULLABLE)
    //          - SQLColAttributeW(SQL_DESC_TYPE_NAME)
    //          - SQLColAttributeW(SQL_COLUMN_LENGTH)
    //          - SQLColAttributeW(SQL_COLUMN_SCALE)
    //      - Until SQLFetch returns SQL_NO_DATA
    //          - SQLFetch()
    //          - For columns 1 to {numCols}
    //              - SQLGetData({colIndex}, {defaultCtoSqlType})
    //      - SQLMoreResults()
    //      -SQLFreeHandle(SQL_HANDLE_STMT)
    #[test]
    fn test_table_listing() {
        let env_handle: HEnv = setup();
        let (conn_handle, _, _, _) = power_bi_connect(env_handle);
        let mut stmt: Handle = null_mut();
        let str_len_ptr = &mut 0;
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
            let column_count_ptr = &mut 0;
            let mut table_view: Vec<u16> = "TABLE,VIEW".encode_utf16().collect();
            table_view.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLTablesW(stmt as HStmt, null_mut(), 0, null_mut(), 0, null_mut(), 0, table_view.as_ptr(), 11),
                "{}",
                get_sql_diagnostics(HandleType::Env, env_handle as Handle)
            );

            // SQLGetFunctions is not available through odbc_sys
            // SQLGetFunctions(SQL_API_SQLFETCHSCROLL)

            test_get_info!(
                conn_handle,
                InfoType::GetDataExtensions,
                2,
                DataType::USmallInt
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLNumResultCols(stmt as HStmt, column_count_ptr)
            );
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
            let expected_col_attr_string_lengths = [
                &[0, 0, 18, 18, 12, 12, 12],
                &[12, 12, 22, 22, 12, 12, 12],
                &[12, 12, 20, 20, 12, 12, 12],
                &[12, 12, 20, 20, 12, 12, 12],
                &[12, 12, 14, 14, 12, 12, 12]
            ];
            for col_num in 0..*column_count_ptr {
                let mut field_num : usize = 0;
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
                    assert_eq!(
                        expected_col_attr_string_lengths[col_num as usize][field_num],
                        *str_len_ptr,
                        "mismatch for string_length_ptr value for row:{field_num} col:{col_num}"
                    );
                    field_num += 1;
                });
            }
            let expected_get_data_string_lengths = [
                &[32, -1, 14, 10, 0],
                &[32, -1, 6, 10, 0],
                &[36, -1, 18, 10, 0],
                &[8, -1, 14, 10, 0],
                &[8, -1, 10, 10, 0],
                &[8, -1, 14, 10, 0],
            ];
            let mut successful_fetch_count = 0;
            let str_len_ptr = &mut 0;
            let mut row_num = 0;
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

                        for col_num in 1..5 {
                            assert_eq!(
                                SqlReturn::SUCCESS,
                                SQLGetData(
                                    stmt as HStmt,
                                    col_num,
                                    odbc_sys::CDataType::WChar,
                                    output_buffer as Pointer,
                                    BUFFER_LENGTH as Len,
                                    str_len_ptr
                                ),
                                "{}",
                                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                            );
                            assert_eq!(
                                expected_get_data_string_lengths[row_num][(col_num - 1) as usize],
                                *str_len_ptr,
                                "mismatch for string_length_ptr value for row:{row_num} col:{col_num}"
                            );
                        }

                        // REMARKS columns are an empty string, NO_DATA is returned
                        assert_eq!(
                            SqlReturn::NO_DATA,
                            SQLGetData(
                                stmt as HStmt,
                                5,
                                odbc_sys::CDataType::WChar,
                                output_buffer as Pointer,
                                BUFFER_LENGTH as Len,
                                str_len_ptr
                            ),
                            "{}",
                            get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
                        );
                        assert_eq!(
                            expected_get_data_string_lengths[row_num][4],
                            *str_len_ptr,
                            "mismatch for string_length_ptr value for row:{row_num} col:4"
                        );

                        row_num += 1;
                    }
                    // break if SQLFetch returns SQL_NO_DATA
                    _ => break,
                }
            }

            assert_eq!(
                6, successful_fetch_count,
                "Expected 6 successful calls to SQLFetch, got {successful_fetch_count}."
            );

            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt as HStmt));
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Stmt, stmt as Handle),
                "{}",
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle)
            );
        }
    }
}
