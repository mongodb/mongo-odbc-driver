#![allow(
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

use crate::SQLSetEnvAttr;
use crate::{
    handles::definitions::{Env, EnvState, MongoHandle},
    SQLGetDiagRecW, SQLGetEnvAttr,
};
use definitions::{
    AttrConnectionPooling, AttrCpMatch, AttrOdbcVersion, EnvironmentAttribute, HEnv, HandleType,
    Integer, Pointer, SqlBool, SqlReturn,
};
use std::{collections::BTreeMap, mem::size_of};

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
            SQLGetEnvAttr(
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
                    SQLSetEnvAttr(handle as HEnv, attr, value, 0)
                );
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetEnvAttr(
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
                    AttrOdbcVersion::SQL_OV_ODBC2 as i32 => SqlReturn::SUCCESS,
                    AttrOdbcVersion::SQL_OV_ODBC3 as i32 => SqlReturn::SUCCESS,
                    AttrOdbcVersion::SQL_OV_ODBC3_80 as i32 => SqlReturn::SUCCESS,
                    1 => SqlReturn::ERROR // Some number other than 2, 3 and 380
                },
                AttrOdbcVersion::SQL_OV_ODBC3_80 as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS,
                map! {
                    SqlBool::SQL_TRUE as i32 => SqlReturn::SUCCESS,
                    SqlBool::SQL_FALSE as i32 => SqlReturn::ERROR
                },
                SqlBool::SQL_TRUE as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_CONNECTION_POOLING,
                map! {
                    AttrConnectionPooling::SQL_CP_OFF as i32 => SqlReturn::SUCCESS,
                    AttrConnectionPooling::SQL_CP_ONE_PER_HENV as i32 => SqlReturn::SUCCESS_WITH_INFO,
                    AttrConnectionPooling::SQL_CP_ONE_PER_DRIVER as i32 => SqlReturn::SUCCESS_WITH_INFO,
                    AttrConnectionPooling:: SQL_CP_DRIVER_AWARE as i32 => SqlReturn::SUCCESS_WITH_INFO,
                },
                AttrConnectionPooling::SQL_CP_OFF as i32,
            );

            get_set_env_attr(
                env_handle,
                EnvironmentAttribute::SQL_ATTR_CP_MATCH,
                map! {
                    AttrCpMatch:: SQL_CP_STRICT_MATCH as i32 => SqlReturn::SUCCESS,
                    AttrCpMatch:: SQL_CP_RELAXED_MATCH as i32 => SqlReturn::SUCCESS_WITH_INFO,
                },
                AttrCpMatch::SQL_CP_STRICT_MATCH as i32,
            );

            // SQLGetEnvAttr where value_ptr is null
            let string_length_ptr = &mut 0;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetEnvAttr(
                    env_handle as *mut _,
                    EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS as i32,
                    std::ptr::null_mut(),
                    0,
                    string_length_ptr
                )
            );
            assert_eq!(0, *string_length_ptr);

            // SQLGetEnvAttr where string_length_ptr is null
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetEnvAttr(
                    env_handle as *mut _,
                    EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS as i32,
                    std::ptr::null_mut(),
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
        use cstr::WideChar;
        unsafe {
            let handle: *mut _ = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLSetEnvAttr(
                    handle as HEnv,
                    EnvironmentAttribute::SQL_ATTR_CP_MATCH as i32,
                    AttrCpMatch::SQL_CP_RELAXED_MATCH as i32 as Pointer,
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
                    HandleType::SQL_HANDLE_ENV,
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
                cstr::from_widechar_ref_lossy(&*(sql_state as *const [WideChar; 6]))
            );
            assert_eq!(
             "[MongoDB][API] Invalid value for attribute SQL_ATTR_CP_MATCH, changed to SQL_CP_STRICT_MATCH\0",
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 93]))
            );
        }
    }
}
