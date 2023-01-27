use crate::map;
use crate::{
    api::definitions::*,
    handles::definitions::{Env, EnvState, MongoHandle},
    SQLGetDiagRecW, SQLGetEnvAttrW, SQLSetEnvAttrW,
};
use odbc_sys::{HEnv, HandleType, Integer, Pointer, SqlReturn};
use std::{collections::BTreeMap, ffi::c_void, mem::size_of};

const OPTIONAL_VALUE_CHANGED: &str = "01S02\0";

fn get_set_env_attr(
    handle: *mut MongoHandle,
    attribute: EnvironmentAttribute,
    value_map: BTreeMap<i32, SqlReturn>,
    default_value: i32,
) {
    let attr_buffer = Box::into_raw(Box::new(0));
    let string_length_ptr = &mut 0;
    let attr = attribute as i32;

    unsafe {
        // Test the environment attribute's default value
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetEnvAttrW(
                handle as *mut _,
                attr,
                attr_buffer as Pointer,
                0,
                string_length_ptr
            )
        );

        assert_eq!(default_value, *(attr_buffer as *const _));
        // All environment attributes are represented numerically
        assert_eq!(size_of::<Integer>() as i32, *string_length_ptr);

        value_map
            .into_iter()
            .for_each(|(discriminant, expected_return)| {
                let value = discriminant as Pointer;
                assert_eq!(
                    expected_return,
                    SQLSetEnvAttrW(handle as HEnv, attr, value, 0)
                );
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetEnvAttrW(
                        handle as *mut _,
                        attr,
                        attr_buffer as Pointer,
                        0,
                        string_length_ptr
                    )
                );
                match expected_return {
                    SqlReturn::SUCCESS => {
                        assert_eq!(discriminant, *(attr_buffer as *const _))
                    }
                    _ => {
                        assert_eq!(default_value, *(attr_buffer as *const _))
                    }
                };
                assert_eq!(size_of::<Integer>() as i32, *string_length_ptr);
            });

        let _ = Box::from_raw(attr_buffer);
    }
}

mod unit {
    use super::*;
    // test_env_attr tests SQLGetEnvAttr and SQLSetEnvAttr with every
    // environment attribute value.
    #[test]
    fn test_env_attr() {
        unsafe {
            use crate::map;
            let env_handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_ODBC_VERSION,
                map! {
                    OdbcVersion::Odbc3 as i32 => SqlReturn::SUCCESS,
                    OdbcVersion::Odbc3_80 as i32 => SqlReturn::SUCCESS,
                    2 => SqlReturn::ERROR // Some number other than 3 and 380
                },
                OdbcVersion::Odbc3_80 as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS,
                map! {
                    SqlBool::True as i32 => SqlReturn::SUCCESS,
                    SqlBool::False as i32 => SqlReturn::ERROR
                },
                SqlBool::True as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_CONNECTION_POOLING,
                map! {
                    ConnectionPooling::Off as i32 => SqlReturn::SUCCESS,
                    ConnectionPooling::OnePerHEnv as i32 => SqlReturn::SUCCESS_WITH_INFO,
                    ConnectionPooling::OnePerDriver as i32 => SqlReturn::SUCCESS_WITH_INFO,
                    ConnectionPooling::DriverAware as i32 => SqlReturn::SUCCESS_WITH_INFO,
                },
                ConnectionPooling::Off as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_CP_MATCH,
                map! {
                    CpMatch::Strict as i32 => SqlReturn::SUCCESS,
                    CpMatch::Relaxed as i32 => SqlReturn::SUCCESS_WITH_INFO,
                },
                CpMatch::Strict as i32,
            );

            // SQLGetEnvAttrW where value_ptr is null
            let string_length_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetEnvAttrW(
                    env_handle as *mut _,
                    EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS as i32,
                    std::ptr::null_mut() as *mut c_void,
                    0,
                    string_length_ptr
                )
            );
            assert_eq!(0, *string_length_ptr);

            // SQLGetEnvAttrW where string_length_ptr is null
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetEnvAttrW(
                    env_handle as *mut _,
                    EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS as i32,
                    std::ptr::null_mut() as *mut c_void,
                    0,
                    std::ptr::null_mut()
                )
            );
        }
    }

    // optional_value_changed tests functions that return the SQL state
    // 01S02: Optional value changed.
    #[test]
    fn test_optional_value_changed() {
        use widechar::WideChar;
        unsafe {
            let handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLSetEnvAttrW(
                    handle as HEnv,
                    EnvironmentAttribute::SQL_ATTR_CP_MATCH as i32,
                    CpMatch::Relaxed as i32 as Pointer,
                    0
                )
            );

            let mut sql_state: [WideChar; 6] = [0; 6];
            let sql_state = &mut sql_state as *mut WideChar;
            let mut message_text: [WideChar; 93] = [0; 93];
            let message_text = &mut message_text as *mut WideChar;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagRecW(
                    HandleType::Env,
                    handle as *mut _,
                    1,
                    sql_state,
                    &mut 0,
                    message_text,
                    93,
                    &mut 0
                )
            );
            assert_eq!(
                OPTIONAL_VALUE_CHANGED,
                widechar::from_widechar_ref_lossy(&*(sql_state as *const [WideChar; 6]))
            );
            assert_eq!(
             "[MongoDB][API] Invalid value for attribute SQL_ATTR_CP_MATCH, changed to SQL_CP_STRICT_MATCH\0",
                widechar::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 93]))
            );
        }
    }
}
