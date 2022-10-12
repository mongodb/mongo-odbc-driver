mod unit {
    use std::sync::RwLock;
    use odbc_sys::{ConnectionAttribute, HDbc, Integer, Pointer, SqlReturn};
    use crate::handles::definitions::{Connection, ConnectionState, MongoHandle};
    use crate::SQLGetConnectAttrW;
    use crate::util::input_wtext_to_string;

    // Test getting CurrentCatalog attribute from the Connection
    // when none of the attributes are explicitly set.
    #[test]
    fn get_string_attrs_default() {
        unsafe {
            let mut conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
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
            assert_eq!("test".to_string(), input_wtext_to_string(value_ptr as *const _, *out_length as usize))
        }
    }

    // Test getting numeric attributes from the Connection
    // when none of the attributes are explicitly set.
    #[test]
    fn get_numeric_attrs_default() {
        unsafe {
            let mut conn = Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
            let mongo_handle: *mut _ = &mut MongoHandle::Connection(RwLock::new(conn));

            let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let out_length = &mut 10;

            for (attr, expected) in [
                (ConnectionAttribute::LoginTimeout, 0),
                (ConnectionAttribute::ConnectionTimeout, 0)
            ] {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        mongo_handle as *mut _,
                        attr,
                        value_ptr,
                        0,
                        out_length
                    )
                );
                assert_eq!(expected, *value_ptr) // TODO: how to compare this as an int
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
                (ConnectionAttribute::ConnectionTimeout, 24u32)
            ] {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetConnectAttrW(
                        mongo_handle as *mut _,
                        attr,
                        value_ptr,
                        0,
                        out_length
                    )
                );
                assert_eq!(expected, *value_ptr) // TODO: how to compare this as an int
            }
        }
    }

    // TODO: test invalid attribute
    // TODO: test setting
}