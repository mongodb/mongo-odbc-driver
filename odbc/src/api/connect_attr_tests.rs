mod unit {
    use crate::handles::definitions::ConnectionAttributes;
    use crate::{
        api::data::input_wtext_to_string,
        errors::ODBCError,
        handles::definitions::{Connection, ConnectionState, MongoHandle},
        SQLGetConnectAttrW, SQLSetConnectAttrW,
    };
    use odbc_sys::{ConnectionAttribute, Integer, Pointer, SqlReturn, UInteger};
    use std::sync::RwLock;

    mod get {
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
                    let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

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
                }
            }
        };
    }

        unsafe fn modify_string_attr(value_ptr: Pointer, out_length: usize) -> String {
            input_wtext_to_string(value_ptr as *const _, out_length)
        }

        unsafe fn modify_numeric_attr(value_ptr: Pointer, _: usize) -> u32 {
            *(value_ptr as *mut UInteger)
        }

        test_get_attr!(
            current_catalog_default,
            attribute = ConnectionAttribute::CurrentCatalog,
            expected_sql_return = SqlReturn::NO_DATA,
        );

        test_get_attr!(
            current_catalog,
            attribute = ConnectionAttribute::CurrentCatalog,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = Box::new(ConnectionAttributes {
                current_catalog: Some("test".to_string()),
                ..Default::default()
            }),
            buffer_length = 5,
            expected_length = 4,
            expected_value = "test".to_string(),
            actual_value_modifier = modify_string_attr,
        );

        test_get_attr!(
            connection_timeout_default,
            attribute = ConnectionAttribute::ConnectionTimeout,
            expected_sql_return = SqlReturn::SUCCESS,
            expected_value = 0u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            connection_timeout,
            attribute = ConnectionAttribute::ConnectionTimeout,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = Box::new(ConnectionAttributes {
                connection_timeout: Some(42),
                ..Default::default()
            }),
            expected_value = 42u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            login_timeout_default,
            attribute = ConnectionAttribute::LoginTimeout,
            expected_sql_return = SqlReturn::SUCCESS,
            expected_value = 0u32,
            actual_value_modifier = modify_numeric_attr,
        );

        test_get_attr!(
            login_timeout,
            attribute = ConnectionAttribute::LoginTimeout,
            expected_sql_return = SqlReturn::SUCCESS,
            initial_attrs = Box::new(ConnectionAttributes {
                login_timeout: Some(42),
                ..Default::default()
            }),
            expected_value = 42u32,
            actual_value_modifier = modify_numeric_attr,
        );
    }

    // Test setting LoginTimeout attribute.
    #[test]
    fn set_login_timeout() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let login_timeout_value: UInteger = 42u32;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::LoginTimeout,
                    login_timeout_value as Pointer,
                    0,
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.read().unwrap().attributes;
            assert_eq!(attributes.login_timeout, Some(42));
        }
    }

    const UNSUPPORTED_ATTRS: [ConnectionAttribute; 19] = [
        ConnectionAttribute::AsyncEnable,
        ConnectionAttribute::AccessMode,
        ConnectionAttribute::AutoCommit,
        ConnectionAttribute::Trace,
        ConnectionAttribute::TraceFile,
        ConnectionAttribute::TranslateLib,
        ConnectionAttribute::TranslateOption,
        ConnectionAttribute::TxnIsolation,
        ConnectionAttribute::OdbcCursors,
        ConnectionAttribute::QuietMode,
        ConnectionAttribute::PacketSize,
        ConnectionAttribute::DisconnectBehaviour,
        ConnectionAttribute::AsyncDbcFunctionsEnable,
        ConnectionAttribute::AsyncDbcEvent,
        ConnectionAttribute::EnlistInDtc,
        ConnectionAttribute::EnlistInXa,
        ConnectionAttribute::ConnectionDead,
        ConnectionAttribute::AutoIpd,
        ConnectionAttribute::MetadataId,
    ];

    // Test getting unsupported attributes.
    #[test]
    fn get_unsupported_attr() {
        unsafe {
            for attr in UNSUPPORTED_ATTRS {
                let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLGetConnectAttrW(
                        mongo_handle as *mut _,
                        attr,
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
                &[
                    ConnectionAttribute::CurrentCatalog,
                    ConnectionAttribute::ConnectionTimeout,
                ][..],
            ]
            .concat()
            {
                let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

                assert_eq!(
                    SqlReturn::ERROR,
                    SQLSetConnectAttrW(
                        mongo_handle as *mut _,
                        attr,
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
        let errors = &conn_handle.read().unwrap().errors;
        assert_eq!(1, errors.len());
        let actual_err = errors.first().unwrap();
        match actual_err {
            ODBCError::UnsupportedConnectionAttribute(actual_attr) => {
                assert_eq!(attr, *actual_attr)
            }
            _ => panic!("unexpected err: {:?}", actual_err),
        }
    }
}
