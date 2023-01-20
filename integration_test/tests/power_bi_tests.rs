mod common;

mod integration {
    use crate::common::{
        generate_default_connection_str, get_sql_diagnostics, sql_return_to_string,
    };
    use odbc::ffi::SQL_NTS;
    use odbc_sys::{
        AttrConnectionPooling, AttrOdbcVersion, ConnectionAttribute, Desc, DriverConnectOption,
        EnvironmentAttribute, HDbc, HEnv, HStmt, Handle, HandleType, InfoType, Pointer,
        SQLAllocHandle, SQLColAttributeW, SQLDriverConnectW, SQLExecDirectW, SQLFetch,
        SQLFreeHandle, SQLGetData, SQLGetInfoW, SQLMoreResults, SQLNumResultCols,
        SQLSetConnectAttrW, SQLSetEnvAttr, SmallInt, SqlReturn,
    };
    use std::ptr::null_mut;
    use std::slice;

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
            let mut in_connection_string_encoded: Vec<u16> =
                in_connection_string.encode_utf16().collect();
            in_connection_string_encoded.push(0);

            let str_len_ptr = &mut 0;
            const BUFFER_LENGTH: SmallInt = 300;
            let out_connection_string_buff = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

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
            out_connection_string = String::from_utf16_lossy(slice::from_raw_parts(
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

            println!(
                "Input connection string = {}\nLength is {}",
                in_connection_string, input_len
            );
            println!(
                "Output connection string = {}\nLength is {}",
                out_connection_string, output_len
            );
            // The output string should be the same as the input string except with extra curly braces around the driver name
            assert_eq!(input_len, output_len, "Expect that both connection the input connection string and ouptput connection string have the same length but input string length is {} and output string length is {}",input_len, output_len);

            let str_len_ptr = &mut 0;
            const BUFFER_LENGTH: SmallInt = 300;
            let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

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

            let mut outcome = SQLGetInfoW(
                conn_handle as HDbc,
                InfoType::DbmsName,
                output_buffer as Pointer,
                BUFFER_LENGTH,
                str_len_ptr,
            );
            assert_eq!(
                SqlReturn::SUCCESS,
                outcome,
                "Expected {}, got {}. Diagnostic message is: {}",
                sql_return_to_string(SqlReturn::SUCCESS),
                sql_return_to_string(outcome),
                get_sql_diagnostics(HandleType::Env, env_handle as Handle)
            );
            println!(
                "DBMS name = {}\nLength is {}",
                String::from_utf16_lossy(slice::from_raw_parts(
                    output_buffer,
                    *str_len_ptr as usize
                )),
                *str_len_ptr
            );

            outcome = SQLGetInfoW(
                conn_handle as HDbc,
                InfoType::DbmsVer,
                output_buffer as Pointer,
                BUFFER_LENGTH,
                str_len_ptr,
            );
            assert_eq!(
                SqlReturn::SUCCESS,
                outcome,
                "Expected {}, got {}. Diagnostic message is: {}",
                sql_return_to_string(SqlReturn::SUCCESS),
                sql_return_to_string(outcome),
                get_sql_diagnostics(HandleType::Env, env_handle as Handle)
            );
            println!(
                "DBMS version = {}\nLength is {}",
                String::from_utf16_lossy(slice::from_raw_parts(
                    output_buffer,
                    *str_len_ptr as usize
                )),
                *str_len_ptr
            );
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
                SQLExecDirectW(stmt as HStmt, query.as_ptr(), SQL_NTS as i32)
            );

            // SQLGetFunctions is not available through odbc_sys
            /*
            SQLGetFunctions(SQL_API_SQLFETCHSCROLL)
            */

            let str_len_ptr = &mut 0;
            const BUFFER_LENGTH: SmallInt = 300;
            let output_buffer = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetInfoW(
                    conn_handle,
                    InfoType::GetDataExtensions,
                    output_buffer as Pointer,
                    BUFFER_LENGTH,
                    str_len_ptr
                )
            );

            let column_count_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLNumResultCols(stmt as HStmt, column_count_ptr)
            );
            let numeric_attribute_ptr = &mut 0;
            const FIELD_TYPES: [Desc; 4] = [
                Desc::ConciseType,
                Desc::Unsigned,
                // SQL_COLUMN_NAME,
                Desc::Unsigned,
                Desc::TypeName,
                // SQL_COLUMN_LENGTH,
                // SQL_COLUMN_SCALE,
            ];
            for col_num in 0..*column_count_ptr {
                FIELD_TYPES.iter().for_each(|field_type| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLColAttributeW(
                            stmt as HStmt,
                            (col_num + 1) as u16,
                            *field_type,
                            output_buffer as Pointer,
                            BUFFER_LENGTH,
                            str_len_ptr,
                            numeric_attribute_ptr
                        )
                    );
                });
            }
            loop {
                let result = SQLFetch(stmt as HStmt);
                assert!(result == SqlReturn::SUCCESS || result == SqlReturn::NO_DATA);
                match result {
                    SqlReturn::SUCCESS => {
                        assert_eq!(
                            SqlReturn::SUCCESS,
                            SQLGetData(
                                stmt as HStmt,
                                1,
                                odbc_sys::CDataType::SLong,
                                output_buffer as Pointer,
                                BUFFER_LENGTH as isize,
                                &mut 0
                            )
                        );
                        assert_eq!(
                            SqlReturn::SUCCESS,
                            SQLGetData(
                                stmt as HStmt,
                                2,
                                odbc_sys::CDataType::Char,
                                output_buffer as Pointer,
                                BUFFER_LENGTH as isize,
                                &mut 0
                            )
                        );
                    }
                    // break if SQLFetch returns SQL_NO_DATA
                    _ => break,
                }
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
}
