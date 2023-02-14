use crate::{
    api::data::input_wtext_to_string,
    definitions::*,
    handles::definitions::{Connection, ConnectionState, MongoHandle},
    SQLGetInfoW,
};
use odbc_sys::{Pointer, SmallInt, SqlReturn, UInteger, USmallInt};
use widechar::WideChar;

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

                let _ = Box::from_raw(value_ptr as *mut USmallInt);
            }
        }
    }
}

macro_rules! test_get_info_expect_u32_zero {
    ($func_name:ident, info_type = $info_type:expr) => {
        test_get_info!(
            $func_name,
            info_type = $info_type,
            expected_sql_return = SqlReturn::SUCCESS,
            expected_length = std::mem::size_of::<u32>() as i16,
            expected_value = 0u32,
            actual_value_modifier = modify_u32_value,
        );
    };
}

unsafe fn modify_string_value(value_ptr: Pointer, out_length: usize) -> String {
    input_wtext_to_string(
        value_ptr as *const _,
        out_length / std::mem::size_of::<WideChar>(),
    )
}

unsafe fn modify_string_value_from_runes(value_ptr: Pointer, out_length: usize) -> String {
    input_wtext_to_string(value_ptr as *const _, out_length)
}

unsafe fn modify_u32_value(value_ptr: Pointer, _: usize) -> u32 {
    *(value_ptr as *mut UInteger)
}

unsafe fn modify_u16_value(value_ptr: Pointer, _: usize) -> u16 {
    *(value_ptr as *mut USmallInt)
}

mod unit {
    use crate::util::format_driver_version;

    use super::*;
    use constants::{DBMS_NAME, DRIVER_NAME, ODBC_VERSION};
    use std::mem::size_of;
    use widechar::WideChar;

    test_get_info!(
        driver_name,
        info_type = InfoType::SQL_DRIVER_NAME as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 40 * size_of::<WideChar>() as i16,
        expected_length = 39 * size_of::<WideChar>() as i16,
        expected_value = DRIVER_NAME,
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        driver_ver,
        info_type = InfoType::SQL_DRIVER_VER as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 11 * size_of::<WideChar>() as i16,
        expected_length = 10 * size_of::<WideChar>() as i16,
        expected_value = format_driver_version(
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
        ),
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        driver_odbc_ver,
        info_type = InfoType::SQL_DRIVER_ODBC_VER as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 6,
        expected_length = 5,
        expected_value = ODBC_VERSION,
        actual_value_modifier = modify_string_value_from_runes,
    );

    test_get_info!(
        search_pattern_escape,
        info_type = InfoType::SQL_SEARCH_PATTERN_ESCAPE as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = size_of::<WideChar>() as i16,
        expected_length = 0,
        expected_value = "",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        dbms_name,
        info_type = InfoType::SQL_DBMS_NAME as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 14 * size_of::<WideChar>() as i16,
        expected_length = 13 * size_of::<WideChar>() as i16,
        expected_value = DBMS_NAME,
        actual_value_modifier = modify_string_value,
    );

    // DbmsVer must be an integration test since it must connect to ADL to get the version

    test_get_info!(
        concat_null_behavior,
        info_type = InfoType::SQL_CONCAT_NULL_BEHAVIOR as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        identifier_quote_char,
        info_type = InfoType::SQL_IDENTIFIER_QUOTE_CHAR as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = "`",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        owner_term,
        info_type = InfoType::SQL_OWNER_TERM as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = size_of::<WideChar>() as i16,
        expected_length = 0,
        expected_value = "",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        catalog_name_separator,
        info_type = InfoType::SQL_CATALOG_NAME_SEPARATOR as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = ".",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        catalog_term,
        info_type = InfoType::SQL_CATALOG_TERM as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 9 * size_of::<WideChar>() as i16,
        expected_length = 8 * size_of::<WideChar>() as i16,
        expected_value = "database",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        convert_functions,
        info_type = InfoType::SQL_CONVERT_FUNCTIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 2u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        numeric_functions,
        info_type = InfoType::SQL_NUMERIC_FUNCTIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_NUM_ABS
            | SQL_FN_NUM_CEILING
            | SQL_FN_NUM_COS
            | SQL_FN_NUM_FLOOR
            | SQL_FN_NUM_LOG
            | SQL_FN_NUM_MOD
            | SQL_FN_NUM_SIN
            | SQL_FN_NUM_SQRT
            | SQL_FN_NUM_TAN
            | SQL_FN_NUM_DEGREES
            | SQL_FN_NUM_POWER
            | SQL_FN_NUM_RADIANS
            | SQL_FN_NUM_ROUND,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        string_functions,
        info_type = InfoType::SQL_STRING_FUNCTIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_STR_CONCAT
            | SQL_FN_STR_LENGTH
            | SQL_FN_STR_SUBSTRING
            | SQL_FN_STR_BIT_LENGTH
            | SQL_FN_STR_CHAR_LENGTH
            | SQL_FN_STR_CHARACTER_LENGTH
            | SQL_FN_STR_OCTET_LENGTH
            | SQL_FN_STR_POSITION,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info_expect_u32_zero!(
        system_functions,
        info_type = InfoType::SQL_SYSTEM_FUNCTIONS as u16
    );

    test_get_info!(
        timedate_functions,
        info_type = InfoType::SQL_TIMEDATE_FUNCTIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_TD_CURRENT_TIMESTAMP | SQL_FN_TD_EXTRACT,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info_expect_u32_zero!(
        convert_big_int,
        info_type = InfoType::SQL_CONVERT_BIGINT as u16
    );

    test_get_info_expect_u32_zero!(
        convert_decimal,
        info_type = InfoType::SQL_CONVERT_DECIMAL as u16
    );

    test_get_info_expect_u32_zero!(
        convert_double,
        info_type = InfoType::SQL_CONVERT_DOUBLE as u16
    );

    test_get_info_expect_u32_zero!(
        convert_float,
        info_type = InfoType::SQL_CONVERT_FLOAT as u16
    );

    test_get_info_expect_u32_zero!(
        convert_integer,
        info_type = InfoType::SQL_CONVERT_INTEGER as u16
    );

    test_get_info_expect_u32_zero!(
        convert_numeric,
        info_type = InfoType::SQL_CONVERT_NUMERIC as u16
    );

    test_get_info_expect_u32_zero!(convert_real, info_type = InfoType::SQL_CONVERT_REAL as u16);

    test_get_info_expect_u32_zero!(
        convert_small_int,
        info_type = InfoType::SQL_CONVERT_SMALLINT as u16
    );

    test_get_info_expect_u32_zero!(
        convert_tiny_int,
        info_type = InfoType::SQL_CONVERT_TINYINT as u16
    );

    test_get_info_expect_u32_zero!(convert_bit, info_type = InfoType::SQL_CONVERT_BIT as u16);

    test_get_info_expect_u32_zero!(convert_char, info_type = InfoType::SQL_CONVERT_CHAR as u16);

    test_get_info_expect_u32_zero!(
        convert_var_char,
        info_type = InfoType::SQL_CONVERT_VARCHAR as u16
    );

    test_get_info_expect_u32_zero!(
        convert_long_var_char,
        info_type = InfoType::SQL_CONVERT_LONGVARCHAR as u16
    );

    test_get_info_expect_u32_zero!(
        convert_w_char,
        info_type = InfoType::SQL_CONVERT_WCHAR as u16
    );

    test_get_info_expect_u32_zero!(
        convert_w_var_char,
        info_type = InfoType::SQL_CONVERT_WVARCHAR as u16
    );

    test_get_info_expect_u32_zero!(
        convert_w_long_var_char,
        info_type = InfoType::SQL_CONVERT_WLONGVARCHAR as u16
    );

    test_get_info_expect_u32_zero!(
        convert_timestamp,
        info_type = InfoType::SQL_CONVERT_TIMESTAMP as u16
    );

    test_get_info_expect_u32_zero!(
        convert_binary,
        info_type = InfoType::SQL_CONVERT_BINARY as u16
    );

    test_get_info_expect_u32_zero!(convert_date, info_type = InfoType::SQL_CONVERT_DATE as u16);

    test_get_info_expect_u32_zero!(convert_time, info_type = InfoType::SQL_CONVERT_TIME as u16);

    test_get_info_expect_u32_zero!(
        convert_var_binary,
        info_type = InfoType::SQL_CONVERT_BINARY as u16
    );

    test_get_info_expect_u32_zero!(
        convert_long_var_binary,
        info_type = InfoType::SQL_CONVERT_LONGVARBINARY as u16
    );

    test_get_info_expect_u32_zero!(convert_guid, info_type = InfoType::SQL_CONVERT_GUID as u16);

    test_get_info!(
        getdata_extensions,
        info_type = InfoType::SQL_GETDATA_EXTENSIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_GD_ANY_COLUMN | SQL_GD_ANY_ORDER,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        column_alias,
        info_type = InfoType::SQL_COLUMN_ALIAS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        group_by,
        info_type = InfoType::SQL_GROUP_BY as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 2u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        order_by_columns_in_select,
        info_type = InfoType::SQL_ORDER_BY_COLUMNS_IN_SELECT as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info_expect_u32_zero!(owner_usage, info_type = InfoType::SQL_OWNER_USAGE as u16);

    test_get_info!(
        catalog_usage,
        info_type = InfoType::SQL_CATALOG_USAGE as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        datasource_ready_only,
        info_type = InfoType::SQL_DATA_SOURCE_READ_ONLY as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        special_characters,
        info_type = InfoType::SQL_SPECIAL_CHARACTERS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 22 * size_of::<WideChar>() as i16,
        expected_length = 21 * size_of::<WideChar>() as i16,
        expected_value = "`\"'.$+-*/|:<>!={}[]()",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        max_columns_in_group_by,
        info_type = InfoType::SQL_MAX_COLUMNS_IN_GROUP_BY as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        max_columns_in_order_by,
        info_type = InfoType::SQL_MAX_COLUMNS_IN_ORDER_BY as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        max_columns_in_select,
        info_type = InfoType::SQL_MAX_COLUMNS_IN_SELECT as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        timedata_add_intervals,
        info_type = InfoType::SQL_TIMEDATE_ADD_INTERVALS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_TSI_SECOND
            | SQL_FN_TSI_MINUTE
            | SQL_FN_TSI_HOUR
            | SQL_FN_TSI_DAY
            | SQL_FN_TSI_WEEK
            | SQL_FN_TSI_MONTH
            | SQL_FN_TSI_QUARTER
            | SQL_FN_TSI_YEAR,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        timedata_diff_intervals,
        info_type = InfoType::SQL_TIMEDATE_DIFF_INTERVALS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_TSI_SECOND
            | SQL_FN_TSI_MINUTE
            | SQL_FN_TSI_HOUR
            | SQL_FN_TSI_DAY
            | SQL_FN_TSI_WEEK
            | SQL_FN_TSI_MONTH
            | SQL_FN_TSI_QUARTER
            | SQL_FN_TSI_YEAR,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        catalog_location,
        info_type = InfoType::SQL_CATALOG_LOCATION as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 1u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        sql_conformance,
        info_type = InfoType::SQL_SQL_CONFORMANCE as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        odbc_interface_conformance,
        info_type = InfoType::SQL_ODBC_INTERFACE_CONFORMANCE as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        sql_92_predicates,
        info_type = InfoType::SQL_SQL92_PREDICATES as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_SP_EXISTS
            | SQL_SP_ISNOTNULL
            | SQL_SP_ISNULL
            | SQL_SP_LIKE
            | SQL_SP_IN
            | SQL_SP_BETWEEN
            | SQL_SP_COMPARISON
            | SQL_SP_QUANTIFIED_COMPARISON,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        sql_92_relational_join_operators,
        info_type = InfoType::SQL_SQL92_RELATIONAL_JOIN_OPERATORS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_SRJO_CROSS_JOIN
            | SQL_SRJO_INNER_JOIN
            | SQL_SRJO_LEFT_OUTER_JOIN
            | SQL_SRJO_RIGHT_OUTER_JOIN,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        aggregate_functions,
        info_type = InfoType::SQL_AGGREGATE_FUNCTIONS as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_AF_AVG
            | SQL_AF_COUNT
            | SQL_AF_MAX
            | SQL_AF_MIN
            | SQL_AF_SUM
            | SQL_AF_DISTINCT
            | SQL_AF_ALL,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        catalog_name,
        info_type = InfoType::SQL_CATALOG_NAME as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2 * size_of::<WideChar>() as i16,
        expected_length = size_of::<WideChar>() as i16,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        max_identifier_len,
        info_type = InfoType::SQL_MAX_IDENTIFIER_LEN as u16,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = u16::MAX,
        actual_value_modifier = modify_u16_value,
    );
}
