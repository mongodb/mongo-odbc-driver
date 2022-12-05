mod common;

mod integration {
    use odbc_sys::{
        HDbc, HEnv, Handle, HandleType, InfoType, Pointer, SQLFreeHandle, SQLGetInfoW, SmallInt,
        SqlReturn,
    };
    use std::slice;

    /// Test PowerBI Setup flow
    #[test]
    fn test_setup() {
        crate::common::setup();
    }

    /// Test PowerBi environment clean-up
    #[test]
    fn test_env_cleanup() {
        // We need a handle to be able to test that freeing the handle work
        let env_handle: HEnv = crate::common::setup();

        unsafe {
            // Verify that freeing the handle is working as expected
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeHandle(HandleType::Env, env_handle as Handle)
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
        let env_handle: HEnv = crate::common::setup();
        let (conn_handle, in_connection_string, out_connection_string, output_len) =
            crate::common::power_bi_connect(env_handle);

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

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetInfoW(
                    conn_handle as HDbc,
                    InfoType::DbmsName,
                    output_buffer as Pointer,
                    BUFFER_LENGTH,
                    str_len_ptr
                )
            );
            println!(
                "DBMS name = {}\nLength is {}",
                String::from_utf16_lossy(slice::from_raw_parts(
                    output_buffer,
                    *str_len_ptr as usize
                )),
                *str_len_ptr
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetInfoW(
                    conn_handle as HDbc,
                    InfoType::DbmsVer,
                    output_buffer as Pointer,
                    BUFFER_LENGTH,
                    str_len_ptr
                )
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
}
