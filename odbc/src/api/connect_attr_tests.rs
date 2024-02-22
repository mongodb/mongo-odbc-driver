mod unit {
    use crate::{
        errors::ODBCError,
        handles::definitions::{Connection, ConnectionAttributes, ConnectionState, MongoHandle},
        util::connection_attribute_to_string,
        SQLGetConnectAttrW, SQLSetConnectAttrW,
    };
    use cstr::input_text_to_string_w;
    use definitions::{ConnectionAttribute, Integer, Pointer, SqlReturn, UInteger};
    use std::sync::RwLock;

    mod get {
        use std::mem::size_of;

        use cstr::WideChar;

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
            initial_attrs = RwLock::new(ConnectionAttributes {
                current_catalog: Some("test".to_string()),
                ..Default::default()
            }),
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
            initial_attrs = RwLock::new(ConnectionAttributes {
                connection_timeout: Some(42),
                ..Default::default()
            }),
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
            initial_attrs = RwLock::new(ConnectionAttributes {
                login_timeout: Some(42),
                ..Default::default()
            }),
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
            assert_eq!(attributes.login_timeout, Some(42));
        }
    }

    // Test setting the current catalog attribute.
    #[test]
    fn set_current_catalog() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);

            let current_catalog_value = "test";
            let current_catalog_ptr = cstr::to_widechar_ptr(current_catalog_value);

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG as i32,
                    current_catalog_ptr.0 as *mut _,
                    current_catalog_ptr.1.len() as i32
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.attributes.read().unwrap();
            assert_eq!(attributes.current_catalog, Some("test".to_string()));
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
