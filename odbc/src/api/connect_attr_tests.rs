mod unit {
    use crate::{
        api::definitions::ConnectionAttribute,
        errors::ODBCError,
        handles::definitions::{Connection, ConnectionState, MongoHandle},
        util::connection_attribute_to_string,
        SQLAllocHandle, SQLDisconnect, SQLDriverConnectW, SQLGetConnectAttrW, SQLSetConnectAttrW,
    };
    use constants::DRIVER_NAME;
    use cstr::{input_text_to_string_w, WideChar};
    use odbc_sys::{
        DriverConnectOption, Handle, HandleType, Integer, Pointer, SmallInt, SqlReturn, UInteger,
    };
    use std::{env, mem::size_of, ptr::null_mut, sync::RwLock};

    mod get {
        use std::mem::size_of;

        use cstr::WideChar;
        use mongo_odbc_core::ConnectionAttributes;

        use super::*;

        macro_rules! test_get_attr {
        ($func_name:ident,
        attribute = $attribute:expr,
        expected_sql_return = $expected_sql_return:expr,
        $(initial_attrs = $initial_attrs:expr,)?
        $(buffer_length = $buffer_length:expr,)?
        $(expected_length = $expected_length:expr,)?
        $(expected_value = $expected_value:expr,)?
        $(actual_value_modifier = $actual_value_modifier:ident,)?
        ) => {
            #[test]
            fn $func_name() {
                unsafe {
                    let attribute = $attribute;

                    #[allow(unused_mut, unused_assignments)]
                    let mut conn =
                        Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                    $(conn.attributes = $initial_attrs;)?
                    let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);

                    let value_ptr: *mut std::ffi::c_void =
                        Box::into_raw(Box::new([0u8; 40])) as *mut _;
                    let out_length = &mut 10;

                    #[allow(unused_mut, unused_assignments)]
                    let mut buffer_length: Integer = 0;
                    $(buffer_length = $buffer_length;)?

                    assert_eq!(
                        $expected_sql_return,
                        SQLGetConnectAttrW(
                            mongo_handle as *mut _,
                            attribute,
                            value_ptr,
                            buffer_length,
                            out_length,
                        )
                    );

                    $(assert_eq!($expected_length, *out_length);)?
                    $(assert_eq!($expected_value, $actual_value_modifier(value_ptr, *out_length as usize));)?

                    let _ = Box::from_raw(value_ptr as *mut UInteger);
                }
            }
        };
    }

        unsafe fn modify_string_attr(value_ptr: Pointer, out_length: usize) -> String {
            input_text_to_string_w(
                value_ptr as *const _,
                out_length / std::mem::size_of::<WideChar>(),
            )
        }

        unsafe fn modify_numeric_attr(value_ptr: Pointer, _: usize) -> u32 {
            *(value_ptr as *mut UInteger)
        }

        test_get_attr!(
            current_catalog_default,
            attribute = ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
            expected_sql_return = SqlReturn::NO_DATA,
        );

        test_get_attr!(
            current_catalog,
            attribute = ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = RwLock::new(Some(ConnectionAttributes {
                current_catalog: Some("test".to_string()),
                ..Default::default()
            })),
            buffer_length = 5 * size_of::<WideChar>() as i32,
            expected_length = 4 * size_of::<WideChar>() as i32,
            expected_value = "test".to_string(),
            actual_value_modifier = modify_string_attr,
        );

        test_get_attr!(
            connection_timeout_default,
            attribute = ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT as i32,
            expected_sql_return = SqlReturn::SUCCESS,
            expected_length = std::mem::size_of::<u32>() as i32,
            expected_value = 0u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            connection_timeout,
            attribute = ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT as i32,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = RwLock::new(Some(ConnectionAttributes {
                connection_timeout: Some(42),
                ..Default::default()
            })),
            expected_length = std::mem::size_of::<u32>() as i32,
            expected_value = 42u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            login_timeout_default,
            attribute = ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
            expected_sql_return = SqlReturn::SUCCESS,
            expected_length = std::mem::size_of::<u32>() as i32,
            expected_value = 0u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            login_timeout,
            attribute = ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = RwLock::new(Some(ConnectionAttributes {
                login_timeout: Some(42),
                ..Default::default()
            })),
            expected_length = std::mem::size_of::<u32>() as i32,
            expected_value = 42u32,
            actual_value_modifier = modify_numeric_attr,
        );
    }

    // Test setting LoginTimeout attribute.
    #[test]
    fn set_login_timeout() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);

            let login_timeout_value: UInteger = 42u32;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
                    login_timeout_value as Pointer,
                    0,
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.attributes.read().unwrap();
            assert_eq!(attributes.as_ref().unwrap().login_timeout, Some(42));
        }
    }

    #[test]
    fn set_current_catalog() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);
            let database = "mongosql-rs";

            let value_ptr = cstr::to_widechar_ptr(database);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                    value_ptr.0 as Pointer,
                    database.len() as i32,
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.attributes.read().unwrap();
            assert_eq!(
                attributes.as_ref().unwrap().current_catalog,
                Some(database.to_string())
            );
        }
    }

    /// Generate the default connection setting defined for the tests using a connection string
    /// of the form 'Driver={};PWD={};USER={};SERVER={}'.
    /// The default driver is 'MongoDB Atlas SQL ODBC Driver' if not specified.
    /// The default auth db is 'admin' if not specified.
    fn generate_default_connection_str() -> String {
        let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
        let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
        let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");

        let db = env::var("ADF_TEST_LOCAL_DB");
        let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
            Ok(val) => val,
            Err(_e) => DRIVER_NAME.to_string(), //Default driver name
        };

        let mut connection_string =
            format!("Driver={{{driver}}};USER={user_name};PWD={password};SERVER={host};");

        // If a db is specified add it to the connection string
        match db {
            Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val + ";")),
            Err(_e) => (), // Do nothing
        };

        connection_string
    }

    #[test]
    #[ignore]
    fn attributes_hand_off_to_mongo_connection() {
        unsafe {
            let mut env_handle: Handle = null_mut();
            let mut conn_handle: Handle = null_mut();

            let login_timeout_value: UInteger = 42u32;

            let value_ptr = cstr::to_widechar_ptr("mongosql-rs");

            let mut out_connection_string: [WideChar; 64] = [0; 64];
            let out_connection_string = &mut out_connection_string as *mut WideChar;
            let string_length_2 = &mut 0;
            let buffer_length: SmallInt = 65;

            let in_connection_string = generate_default_connection_str();

            let mut in_connection_string_encoded = cstr::to_widechar_vec(&in_connection_string);
            in_connection_string_encoded.push(0);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(
                    HandleType::Env,
                    std::ptr::null_mut(),
                    &mut env_handle as *mut Handle,
                )
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLAllocHandle(HandleType::Dbc, env_handle, &mut conn_handle as *mut Handle)
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    conn_handle as *mut _,
                    ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                    value_ptr.0 as Pointer,
                    "mongosql-rs".len() as i32
                )
            );

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    conn_handle as *mut _,
                    ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
                    login_timeout_value as Pointer,
                    42,
                )
            );

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length,
                    )
                );

                assert_eq!(
                    "mongosql-rs",
                    input_text_to_string_w(
                        return_ptr as *const WideChar,
                        *out_length as usize / size_of::<WideChar>()
                    )
                );

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length
                    )
                );

                assert_eq!(42u32, *(return_ptr as *mut UInteger));

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }

            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLDriverConnectW(
                    conn_handle as *mut _,
                    std::ptr::null_mut(),
                    in_connection_string_encoded.as_ptr(),
                    in_connection_string.len().try_into().unwrap(),
                    out_connection_string,
                    buffer_length,
                    string_length_2,
                    DriverConnectOption::NoPrompt,
                )
            );

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length,
                    )
                );

                assert_eq!(
                    "mongosql-rs",
                    input_text_to_string_w(
                        return_ptr as *const WideChar,
                        *out_length as usize / size_of::<WideChar>()
                    )
                );

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length
                    )
                );

                assert_eq!(42u32, *(return_ptr as *mut UInteger));

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }

            assert_eq!(SqlReturn::SUCCESS, SQLDisconnect(conn_handle as *mut _));

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length,
                    )
                );

                assert_eq!(
                    "mongosql-rs",
                    input_text_to_string_w(
                        return_ptr as *const WideChar,
                        *out_length as usize / size_of::<WideChar>()
                    )
                );

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }

            {
                let return_ptr: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let input_buffer_length = 20 * size_of::<WideChar>() as i32;
                let out_length = &mut 20;

                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        conn_handle as *mut _,
                        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT as i32,
                        return_ptr,
                        input_buffer_length,
                        out_length
                    )
                );

                assert_eq!(42u32, *(return_ptr as *mut UInteger));

                let _ = Box::from_raw(return_ptr as *mut UInteger);
            }
        }
    }

    const UNSUPPORTED_ATTRS: [ConnectionAttribute; 19] = [
        ConnectionAttribute::SQL_ATTR_ASYNC_ENABLE,
        ConnectionAttribute::SQL_ATTR_ACCESS_MODE,
        ConnectionAttribute::SQL_ATTR_AUTOCOMMIT,
        ConnectionAttribute::SQL_ATTR_TRACE,
        ConnectionAttribute::SQL_ATTR_TRACEFILE,
        ConnectionAttribute::SQL_ATTR_TRANSLATE_LIB,
        ConnectionAttribute::SQL_ATTR_TRANSLATE_OPTION,
        ConnectionAttribute::SQL_ATTR_TXN_ISOLATION,
        ConnectionAttribute::SQL_ATTR_ODBC_CURSORS,
        ConnectionAttribute::SQL_ATTR_QUIET_MODE,
        ConnectionAttribute::SQL_ATTR_PACKET_SIZE,
        ConnectionAttribute::SQL_ATTR_DISCONNECT_BEHAVIOR,
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_FUNCTIONS_ENABLE,
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_EVENT,
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_DTC,
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_XA,
        ConnectionAttribute::SQL_ATTR_CONNECTION_DEAD,
        ConnectionAttribute::SQL_ATTR_AUTO_IPD,
        ConnectionAttribute::SQL_ATTR_METADATA_ID,
    ];

    // Test getting unsupported attributes.
    #[test]
    fn get_unsupported_attr() {
        unsafe {
            for attr in UNSUPPORTED_ATTRS {
                let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLGetConnectAttrW(
                        mongo_handle as *mut _,
                        attr as i32,
                        std::ptr::null_mut() as Pointer,
                        0,
                        std::ptr::null_mut()
                    )
                );
                // Check the actual error
                assert_unsupported_connection_attr_error(mongo_handle, attr)
            }
        }
    }

    // Test setting invalid attributes.
    #[test]
    fn set_invalid_attr() {
        unsafe {
            for attr in [
                &UNSUPPORTED_ATTRS[..],
                &[ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT][..],
            ]
            .concat()
            {
                let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);

                assert_eq!(
                    SqlReturn::ERROR,
                    SQLSetConnectAttrW(
                        mongo_handle as *mut _,
                        attr as i32,
                        std::ptr::null_mut() as Pointer,
                        0
                    )
                );
                assert_unsupported_connection_attr_error(mongo_handle, attr)
            }
        }
    }

    // helper to assert actual error returned by Set/GetAttr
    unsafe fn assert_unsupported_connection_attr_error(
        mongo_handle: *mut MongoHandle,
        attr: ConnectionAttribute,
    ) {
        let conn_handle = (*mongo_handle).as_connection().unwrap();
        let errors = &conn_handle.errors.read().unwrap();
        assert_eq!(1, errors.len());
        let actual_err = errors.first().unwrap();
        match actual_err {
            ODBCError::UnsupportedConnectionAttribute(actual_attr) => {
                assert_eq!(connection_attribute_to_string(attr), *actual_attr)
            }
            _ => panic!("unexpected err: {actual_err:?}"),
        }
    }
}
