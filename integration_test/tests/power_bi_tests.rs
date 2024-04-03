mod common;

mod integration {
    use crate::common::{
        fetch_and_get_data, generate_default_connection_str, get_column_attributes,
        get_sql_diagnostics, sql_return_to_string, OutputBuffer, BUFFER_LENGTH,
    };
    use definitions::{
        AttrConnectionPooling, AttrOdbcVersion, CDataType, ConnectionAttribute,
        DriverConnectOption, EnvironmentAttribute, HDbc, HEnv, HStmt, Handle, HandleType, InfoType,
        Pointer, SQLAllocHandle, SQLDriverConnectW, SQLExecDirectW, SQLFreeHandle, SQLGetInfoW,
        SQLSetConnectAttrW, SQLSetEnvAttr, SQLTablesW, SmallInt, SqlReturn, SQL_NTS,
    };

    use cstr::WideChar;
    use std::ptr::null_mut;
    use std::slice;

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
                info_type as u16,
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
                get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, conn_handle as Handle)
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

    /// Setup flow.
    /// This will allocate a new environment handle and set ODBC_VERSION and CONNECTION_POOLING environment attributes.
    /// Setup flow is:
    ///     - SQLAllocHandle(SQL_HANDLE_ENV)
    ///     - SQLSetEnvAttr(SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3)
    ///     - SQLSetEnvAttr(SQL_ATTR_CONNECTION_POOLING, SQL_CP_ONE_PER_HENV)
    fn setup() -> definitions::HEnv {
        let mut env: Handle = null_mut();

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(
                    HandleType::SQL_HANDLE_ENV as i16,
                    null_mut(),
                    &mut env as *mut Handle
                )
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetEnvAttr(
                    env as HEnv,
                    EnvironmentAttribute::SQL_ATTR_ODBC_VERSION as i32,
                    AttrOdbcVersion::SQL_OV_ODBC3.into(),
                    0,
                )
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetEnvAttr(
                    env as HEnv,
                    EnvironmentAttribute::SQL_ATTR_CONNECTION_POOLING as i32,
                    AttrConnectionPooling::SQL_CP_ONE_PER_HENV.into(),
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
    fn power_bi_connect(env_handle: HEnv) -> (definitions::HDbc, String, String, SmallInt) {
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
                    HandleType::SQL_HANDLE_DBC as i16,
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
                        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
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
            const BUFFER_LENGTH: SmallInt = 1024;
            let mut out_connection_string_buff: [WideChar; BUFFER_LENGTH as usize - 1] =
                [0; (BUFFER_LENGTH as usize - 1)];
            let out_connection_string_buff = &mut out_connection_string_buff as *mut WideChar;

            assert_ne!(
                SqlReturn::ERROR,
                SQLDriverConnectW(
                    dbc as HDbc,
                    null_mut(),
                    in_connection_string_encoded.as_ptr(),
                    SQL_NTS as SmallInt,
                    out_connection_string_buff,
                    BUFFER_LENGTH,
                    str_len_ptr,
                    DriverConnectOption::SQL_DRIVER_NO_PROMPT as u16,
                ),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_DBC, dbc)
            );

            output_len = *str_len_ptr;
            // The iodbc driver manager is multiplying the output length by size_of WideChar (u32)
            // for some reason. It is correct when returned from SQLDriverConnectW, but is 4x
            // bigger between return and here.
            if definitions::USING_IODBC {
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
                SQLFreeHandle(HandleType::SQL_HANDLE_ENV as i16, env_handle as Handle),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env_handle as Handle)
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

            // SQL_DRIVER_NAME is not accessible through definitions
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
                InfoType::SQL_DBMS_NAME,
                14 * (std::mem::size_of::<WideChar>() as i16),
                DataType::WChar
            );

            test_get_info!(
                conn_handle,
                InfoType::SQL_DBMS_VER,
                29 * (std::mem::size_of::<WideChar>() as i16),
                DataType::WChar
            );
        }
    }

    // Test PowerBI driver information retrieval
    // This test is limited by the available InfoType values in definitions
    #[test]
    fn test_get_driver_info() {
        let env_handle: HEnv = setup();
        let (conn_handle, _, _, _) = power_bi_connect(env_handle);

        unsafe {
            test_get_info!(
                conn_handle,
                InfoType::SQL_IDENTIFIER_QUOTE_CHAR,
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
                InfoType::SQL_MAX_COLUMNS_IN_ORDER_BY,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::SQL_MAX_IDENTIFIER_LEN,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::SQL_MAX_COLUMNS_IN_GROUP_BY,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::SQL_MAX_COLUMNS_IN_SELECT,
                2,
                DataType::USmallInt
            );
            test_get_info!(
                conn_handle,
                InfoType::SQL_ORDER_BY_COLUMNS_IN_SELECT,
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
                InfoType::SQL_CATALOG_NAME,
                (2 * std::mem::size_of::<WideChar>()) as i16,
                DataType::WChar
            );
            // InfoType::SQL_CATALOG_TERM
            // InfoType::SQL_OWNER_TERM
            // InfoType::SQL_ODBC_INTERFACE_CONFORMANCE
            test_get_info!(
                conn_handle,
                InfoType::SQL_SEARCH_PATTERN_ESCAPE,
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
                InfoType::SQL_SPECIAL_CHARACTERS,
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
                    HandleType::SQL_HANDLE_STMT as i16,
                    conn_handle as *mut _,
                    &mut stmt as *mut Handle
                )
            );

            // SQL_DRIVER_ODBC_VER and SQL_DRIVER_NAME are not available through definitions
            /*
            SQLGetInfoW(SQL_DRIVER_ODBC_VER)
            SQLGetInfoW(SQL_DRIVER_NAME)
            */
            let current_db = cstr::to_widechar_ptr("integration_test");
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    conn_handle,
                    ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                    current_db.0 as *mut _,
                    current_db.1.len() as i32
                )
            );
            let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from example");
            query.push(0);
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), SQL_NTS as SmallInt as i32),
                "{}",
                get_sql_diagnostics(HandleType::SQL_HANDLE_STMT, stmt as Handle)
            );

            // SQLGetFunctions is not available through definitions
            /*
            SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
            */

            //SQLGetInfoW(SQL_GETDATA_EXTENSIONS)
            test_get_info!(
                conn_handle,
                InfoType::SQL_GETDATA_EXTENSIONS,
                2,
                DataType::USmallInt
            );

            //  - SQLNumResultCols()
            //  - For columns 1 to {numCols}
            //      - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
            //      - SQLColAttributeW(SQL_DESC_UNSIGNED)
            //      - SQLColAttributeW(SQL_COLUMN_NAME)
            //      - SQLColAttributeW(SQL_COLUMN_NULLABLE)
            //      - SQLColAttributeW(SQL_DESC_TYPE_NAME)
            //      - SQLColAttributeW(SQL_COLUMN_LENGTH)
            //      - SQLColAttributeW(SQL_COLUMN_SCALE)
            get_column_attributes(stmt, 2);

            //  - Until SQLFetch returns SQL_NO_DATA
            //      - SQLFetch()
            //      - For columns 1 to {numCols}
            //          - SQLGetData({colIndex}, {defaultCtoSqlType})
            //  - SQLMoreResults()
            fetch_and_get_data(
                stmt,
                Some(3),
                vec![SqlReturn::SUCCESS; 2],
                vec![CDataType::SQL_C_SLONG, CDataType::SQL_C_WCHAR],
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
                    HandleType::SQL_HANDLE_STMT as i16,
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
                get_sql_diagnostics(HandleType::SQL_HANDLE_ENV, env_handle as Handle)
            );

            //  - SQLNumResultCols()
            //  - For columns 1 to {numCols}
            //      - SQLColAttributeW(SQL_DESC_CONCISE_TYPE)
            //      - SQLColAttributeW(SQL_DESC_UNSIGNED)
            //      - SQLColAttributeW(SQL_COLUMN_NAME)
            //      - SQLColAttributeW(SQL_COLUMN_NULLABLE)
            //      - SQLColAttributeW(SQL_DESC_TYPE_NAME)
            //      - SQLColAttributeW(SQL_COLUMN_LENGTH)
            //      - SQLColAttributeW(SQL_COLUMN_SCALE)
            get_column_attributes(stmt, 5);

            //  - Until SQLFetch returns SQL_NO_DATA
            //      - SQLFetch()
            //      - For columns 1 to {numCols}
            //          - SQLGetData({colIndex}, {defaultCtoSqlType})
            //  - SQLMoreResults()
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
                vec![CDataType::SQL_C_WCHAR; 5],
            );
        }
    }
}
