mod unit {
    use crate::{
        errors::ODBCError,
        handles::definitions::{Connection, ConnectionState, MongoHandle},
        util::input_wtext_to_string,
        SQLGetConnectAttrW, SQLSetConnectAttrW,
    };
    use odbc_sys::{ConnectionAttribute, Integer, Pointer, SqlReturn, UInteger};
    use std::sync::RwLock;

    // Test getting CurrentCatalog attribute from the Connection
    // when none of the attributes are set.
    #[test]
    fn get_string_attrs_default() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let buffer_length: Integer = 20;
            let out_length = &mut 10;

            assert_eq!(
                SqlReturn::NO_DATA,
                SQLGetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::CurrentCatalog,
                    value_ptr,
                    buffer_length,
                    out_length,
                )
            );
        }
    }

    // Test getting CurrentCatalog attribute from the Connection
    // when it is explicitly set.
    #[test]
    fn get_string_attrs() {
        unsafe {
            let mut conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            conn.attributes.current_catalog = Some("test".to_string());
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let buffer_length: Integer = 20;
            let out_length = &mut 10;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::CurrentCatalog,
                    value_ptr,
                    buffer_length,
                    out_length,
                )
            );
            assert_eq!("test".len() as i32, *out_length);
            assert_eq!(
                "test".to_string(),
                input_wtext_to_string(value_ptr as *const _, *out_length as usize)
            )
        }
    }

    // Test getting numeric attributes from the Connection
    // when none of the attributes are set.
    #[test]
    fn get_numeric_attrs_default() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let out_length = &mut 10;

            for (attr, expected) in [
                (ConnectionAttribute::LoginTimeout, 0),
                (ConnectionAttribute::ConnectionTimeout, 0),
            ] {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(mongo_handle as *mut _, attr, value_ptr, 0, out_length)
                );
                assert_eq!(expected, *(value_ptr as *mut UInteger))
            }
        }
    }

    // Test getting numeric attributes from the Connection
    // when they are explicitly set.
    #[test]
    fn get_numeric_attrs() {
        unsafe {
            let mut conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            conn.attributes.login_timeout = Some(42);
            conn.attributes.connection_timeout = Some(24);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let out_length = &mut 10;

            for (attr, expected) in [
                (ConnectionAttribute::LoginTimeout, 42u32),
                (ConnectionAttribute::ConnectionTimeout, 24u32),
            ] {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(mongo_handle as *mut _, attr, value_ptr, 0, out_length)
                );
                assert_eq!(expected, *(value_ptr as *mut UInteger))
            }
        }
    }

    // Test getting invalid attributes.
    #[test]
    fn get_invalid_attr() {
        unsafe {
            let out_length = &mut 10;

            for attr in [
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
            ] {
                let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLGetConnectAttrW(
                        mongo_handle as *mut _,
                        attr,
                        std::ptr::null_mut() as Pointer,
                        0,
                        out_length
                    )
                );
                // Check the actual
                let conn_handle = (*mongo_handle).as_connection().unwrap();
                let errors = &conn_handle.read().unwrap().errors;
                assert_eq!(1, errors.len());
                let actual_err = errors.first().unwrap();
                match actual_err {
                    ODBCError::InvalidAttrIdentifier(actual_attr) => assert_eq!(attr, *actual_attr),
                    _ => panic!("unexpected err: {:?}", actual_err),
                }
            }
        }
    }

    // Test setting CurrentCatalog attribute.
    #[test]
    fn set_string_attrs() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let mut value = "test".encode_utf16().collect::<Vec<u16>>();
            value.push('\u{0}' as u16);
            let buffer_length: Integer = 4;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::CurrentCatalog,
                    value.as_ptr() as Pointer,
                    buffer_length,
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.read().unwrap().attributes;
            assert_eq!(attributes.current_catalog, Some("test".to_string()));
        }
    }

    // Test setting LoginTimeout and ConnectionTimeout attributes.
    #[test]
    fn set_numeric_attrs() {
        unsafe {
            let conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let login_timeout_value: UInteger = 42u32;
            let connection_timeout_value: UInteger = 24u32;

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::LoginTimeout,
                    login_timeout_value as Pointer,
                    0,
                )
            );
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLSetConnectAttrW(
                    mongo_handle as *mut _,
                    ConnectionAttribute::ConnectionTimeout,
                    connection_timeout_value as Pointer,
                    0,
                )
            );
            let conn_handle = (*mongo_handle).as_connection().unwrap();
            let attributes = &conn_handle.read().unwrap().attributes;
            // We do support setting LoginTimeout
            assert_eq!(attributes.login_timeout, Some(42));
            // We do not support setting ConnectionTimeout
            assert_eq!(attributes.connection_timeout, None);
        }
    }

    // Test setting invalid attributes.
    #[test]
    fn set_invalid_attr() {
        unsafe {
            for attr in [
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
            ] {
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
                // Check the actual
                let conn_handle = (*mongo_handle).as_connection().unwrap();
                let errors = &conn_handle.read().unwrap().errors;
                assert_eq!(1, errors.len());
                let actual_err = errors.first().unwrap();
                match actual_err {
                    ODBCError::InvalidAttrIdentifier(actual_attr) => assert_eq!(attr, *actual_attr),
                    _ => panic!("unexpected err: {:?}", actual_err),
                }
            }
        }
    }
}
