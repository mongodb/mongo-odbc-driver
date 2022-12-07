use crate::api::data::input_wtext_to_string;
use crate::definitions::InfoType;
use crate::handles::definitions::{Connection, ConnectionState, MongoHandle};
use crate::SQLGetInfoW;
use odbc_sys::{Pointer, SmallInt, SqlReturn /*UInteger, USmallInt*/};

macro_rules! test_get_info {
    ($func_name:ident,
    info_type = $info_type:expr,
    expected_sql_return = $expected_sql_return:expr,
    $(buffer_length = $buffer_length:expr,)?
    $(expected_length = $expected_length:expr,)?
    $(expected_value = $expected_value:expr,)?
    $(actual_value_modifier = $actual_value_modifier:ident,)?
    ) => {
        #[test]
        fn $func_name() {
            unsafe {
                let info_type = $info_type;

                #[allow(unused_mut, unused_assignments)]
                let mut conn =
                    Connection::with_state(std::ptr::null_mut(), ConnectionState::Connected);
                let mongo_handle: *mut _ = &mut MongoHandle::Connection(conn);

                let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 100])) as *mut _;
                let out_length: *mut SmallInt = &mut 10;

                #[allow(unused_mut, unused_assignments)]
                let mut buffer_length: SmallInt = 0;
                $(buffer_length = $buffer_length;)?

                // Assert that the actual result matches expected
                assert_eq!(
                    $expected_sql_return,
                    SQLGetInfoW(
                        mongo_handle as *mut _,
                        info_type,
                        value_ptr as Pointer,
                        buffer_length,
                        out_length,
                    )
                );

                // If the expectation is that the function returns successfully,
                // assert that the resulting value and length are correct
                match $expected_sql_return {
                    SqlReturn::SUCCESS => {
                        $(assert_eq!($expected_length, *out_length);)?
                        $(assert_eq!($expected_value, $actual_value_modifier(value_ptr, *out_length as usize));)?
                    },
                    _ => ()
                }

                let _ = Box::from_raw(value_ptr);
            }
        }
    }
}

unsafe fn modify_string_attr(value_ptr: Pointer, out_length: usize) -> String {
    input_wtext_to_string(value_ptr as *const _, out_length)
}

// unsafe fn modify_u32_attr(value_ptr: Pointer, _: usize) -> u32 {
//     *(value_ptr as *mut UInteger)
// }
//
// unsafe fn modify_u16_attr(value_ptr: Pointer, _: usize) -> u16 {
//     *(value_ptr as *mut USmallInt)
// }

mod unit {
    use super::*;

    test_get_info!(
        driver_name,
        info_type = InfoType::DriverName,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 40,
        expected_length = 39,
        expected_value = "MongoDB Atlas SQL interface ODBC Driver",
        actual_value_modifier = modify_string_attr,
    );

    test_get_info!(
        driver_ver,
        info_type = InfoType::DriverVer,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 11,
        expected_length = 10,
        expected_value = "00.01.0000",
        actual_value_modifier = modify_string_attr,
    );

    // TODO: change the InfoType variant to correct name and value
    test_get_info!(
        driver_odbc_ver,
        info_type = InfoType::DriverOdbcVer,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 6,
        expected_length = 5,
        expected_value = "03.08",
        actual_value_modifier = modify_string_attr,
    );
}
