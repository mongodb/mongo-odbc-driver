mod common;

mod integration {
    use crate::common::{generate_default_connection_str, setup};
    use odbc::ffi::SQL_NTS;
    use odbc_sys::{
        ConnectionAttribute, DriverConnectOption, HDbc, HEnv, Handle, HandleType, Pointer,
        SQLAllocHandle, SQLDriverConnectW, SQLFreeHandle, SQLSetConnectAttrW, SmallInt, SqlReturn,
    };
    use std::ptr::null_mut;
    use std::slice;

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
        let env_handle: HEnv = setup();

        unsafe {
            // Allocate a DBC handle
            let mut dbc: Handle = null_mut();
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
            let in_connection_string = generate_default_connection_str();
            let mut in_connection_string_encoded: Vec<u16> =
                in_connection_string.encode_utf16().collect();
            in_connection_string_encoded.push(0);

            let string_length_2 = &mut 0;
            const BUFFER_LENGTH: SmallInt = 300;
            let out_connection_string = &mut [0u16; (BUFFER_LENGTH as usize - 1)] as *mut _;

            assert_ne!(
                SqlReturn::ERROR,
                SQLDriverConnectW(
                    dbc as HDbc,
                    null_mut(),
                    in_connection_string_encoded.as_ptr(),
                    SQL_NTS,
                    out_connection_string,
                    BUFFER_LENGTH,
                    string_length_2,
                    DriverConnectOption::NoPrompt,
                )
            );

            let input_len = in_connection_string.len() as SmallInt;
            let output_len = *string_length_2;
            let outputstr =
                slice::from_raw_parts(out_connection_string, output_len as usize).to_vec();

            println!(
                "Input connection string = {}\nLength is {}",
                in_connection_string, input_len
            );
            println!(
                "Output connection string = {}\nLength is {}",
                String::from_utf16_lossy(&outputstr),
                output_len
            );
            // The output string should be the same as the input string except with extra curly braces around the driver name
            assert_eq!(input_len, output_len, "Expect that both connection the input connection string and ouptput connection string have the same length but input string length is {} and output string length is {}",input_len, output_len);
        }
    }
}
