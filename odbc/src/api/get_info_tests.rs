use crate::{
    api::data::input_wtext_to_string,
    definitions::*,
    handles::definitions::{Connection, ConnectionState, MongoHandle},
    SQLGetInfoW,
};
use odbc_sys::{Pointer, SmallInt, SqlReturn, UInteger, USmallInt};

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
    input_wtext_to_string(value_ptr as *const _, out_length)
}

unsafe fn modify_u32_value(value_ptr: Pointer, _: usize) -> u32 {
    *(value_ptr as *mut UInteger)
}

unsafe fn modify_u16_value(value_ptr: Pointer, _: usize) -> u16 {
    *(value_ptr as *mut USmallInt)
}

mod unit {
    use super::*;
    use constants::{DBMS_NAME, DRIVER_NAME};

    test_get_info!(
        driver_name,
        info_type = InfoType::DriverName,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 40,
        expected_length = 39,
        expected_value = DRIVER_NAME,
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        driver_ver,
        info_type = InfoType::DriverVer,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 11,
        expected_length = 10,
        expected_value = "00.01.0000",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        driver_odbc_ver,
        info_type = InfoType::DriverOdbcVer,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 6,
        expected_length = 5,
        expected_value = "03.08",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        search_pattern_escape,
        info_type = InfoType::SearchPatternEscape,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 1,
        expected_length = 0,
        expected_value = "",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        dbms_name,
        info_type = InfoType::DbmsName,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 14,
        expected_length = 13,
        expected_value = DBMS_NAME,
        actual_value_modifier = modify_string_value,
    );

    // DbmsVer must be an integration test since it must connect to ADL to get the version

    test_get_info!(
        concat_null_behavior,
        info_type = InfoType::ConcatNullBehavior,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        identifier_quote_char,
        info_type = InfoType::IdentifierQuoteChar,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = "`",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        owner_term,
        info_type = InfoType::OwnerTerm,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 1,
        expected_length = 0,
        expected_value = "",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        catalog_name_separator,
        info_type = InfoType::CatalogNameSeparator,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = ".",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        catalog_term,
        info_type = InfoType::CatalogTerm,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 9,
        expected_length = 8,
        expected_value = "database",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        convert_functions,
        info_type = InfoType::ConvertFunctions,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 2u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        numeric_functions,
        info_type = InfoType::NumericFunctions,
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
        info_type = InfoType::StringFunctions,
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

    test_get_info!(
        system_functions,
        info_type = InfoType::SystemFunctions,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 0u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        timedate_functions,
        info_type = InfoType::TimedateFunctions,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_FN_TD_CURRENT_TIMESTAMP | SQL_FN_TD_EXTRACT,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info_expect_u32_zero!(convert_big_int, info_type = InfoType::ConvertBigInt);

    test_get_info_expect_u32_zero!(convert_decimal, info_type = InfoType::ConvertDecimal);

    test_get_info_expect_u32_zero!(convert_double, info_type = InfoType::ConvertDouble);

    test_get_info_expect_u32_zero!(convert_float, info_type = InfoType::ConvertFloat);

    test_get_info_expect_u32_zero!(convert_integer, info_type = InfoType::ConvertInteger);

    test_get_info_expect_u32_zero!(convert_numeric, info_type = InfoType::ConvertNumeric);

    test_get_info_expect_u32_zero!(convert_real, info_type = InfoType::ConvertReal);

    test_get_info_expect_u32_zero!(convert_small_int, info_type = InfoType::ConvertSmallInt);

    test_get_info_expect_u32_zero!(convert_tiny_int, info_type = InfoType::ConvertTinyInt);

    test_get_info_expect_u32_zero!(convert_bit, info_type = InfoType::ConvertBit);

    test_get_info_expect_u32_zero!(convert_char, info_type = InfoType::ConvertChar);

    test_get_info_expect_u32_zero!(convert_var_char, info_type = InfoType::ConvertVarChar);

    test_get_info_expect_u32_zero!(
        convert_long_var_char,
        info_type = InfoType::ConvertLongVarChar
    );

    test_get_info_expect_u32_zero!(convert_w_char, info_type = InfoType::ConvertWChar);

    test_get_info_expect_u32_zero!(convert_w_var_char, info_type = InfoType::ConvertWVarChar);

    test_get_info_expect_u32_zero!(
        convert_w_long_var_char,
        info_type = InfoType::ConvertWLongVarChar
    );

    test_get_info_expect_u32_zero!(convert_timestamp, info_type = InfoType::ConvertTimestamp);

    test_get_info_expect_u32_zero!(convert_binary, info_type = InfoType::ConvertBinary);

    test_get_info_expect_u32_zero!(convert_date, info_type = InfoType::ConvertDate);

    test_get_info_expect_u32_zero!(convert_time, info_type = InfoType::ConvertTime);

    test_get_info_expect_u32_zero!(convert_var_binary, info_type = InfoType::ConvertVarBinary);

    test_get_info_expect_u32_zero!(
        convert_long_var_binary,
        info_type = InfoType::ConvertLongVarBinary
    );

    test_get_info_expect_u32_zero!(convert_guid, info_type = InfoType::ConvertGuid);

    test_get_info!(
        getdata_extensions,
        info_type = InfoType::GetDataExtensions,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = SQL_GD_ANY_COLUMN | SQL_GD_ANY_ORDER,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        column_alias,
        info_type = InfoType::ColumnAlias,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        group_by,
        info_type = InfoType::GroupBy,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 2u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        order_by_columns_in_select,
        info_type = InfoType::OrderByColumnsInSelect,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        owner_usage,
        info_type = InfoType::OwnerUsage,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 0u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        catalog_usage,
        info_type = InfoType::CatalogUsage,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        datasource_ready_only,
        info_type = InfoType::DataSourceReadOnly,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        special_characters,
        info_type = InfoType::SpecialCharacters,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 22,
        expected_length = 21,
        expected_value = "`\"'.$+-*/|:<>!={}[]()",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        max_columns_in_group_by,
        info_type = InfoType::MaxColumnsInGroupBy,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        max_columns_in_order_by,
        info_type = InfoType::MaxColumnsInOrderBy,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        max_columns_in_select,
        info_type = InfoType::MaxColumnsInSelect,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        timedata_add_intervals,
        info_type = InfoType::TimedateAddIntervals,
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
        info_type = InfoType::TimedateDiffIntervals,
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
        info_type = InfoType::CatalogLocation,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 1u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        sql_conformance,
        info_type = InfoType::SqlConformance,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        odbc_interface_conformance,
        info_type = InfoType::OdbcInterfaceConformance,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u32>() as i16,
        expected_value = 1u32,
        actual_value_modifier = modify_u32_value,
    );

    test_get_info!(
        sql_92_predicates,
        info_type = InfoType::Sql92Predicates,
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
        info_type = InfoType::Sql92RelationalJoinOperators,
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
        info_type = InfoType::AggregateFunctions,
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
        return_escape_clause,
        info_type = InfoType::ReturnEscapeClause,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );

    test_get_info!(
        catalog_name,
        info_type = InfoType::CatalogName,
        expected_sql_return = SqlReturn::SUCCESS,
        buffer_length = 2,
        expected_length = 1,
        expected_value = "Y",
        actual_value_modifier = modify_string_value,
    );

    test_get_info!(
        max_identifier_len,
        info_type = InfoType::MaxIdentifierLen,
        expected_sql_return = SqlReturn::SUCCESS,
        expected_length = std::mem::size_of::<u16>() as i16,
        expected_value = 0u16,
        actual_value_modifier = modify_u16_value,
    );
}
