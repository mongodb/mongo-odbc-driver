use crate::{
    handles::definitions::{MongoHandle, Statement, StatementState},
    SQLColAttributeW, SQLDescribeColW,
};
use mongo_odbc_core::MongoFields;
use odbc_sys::{Desc, Nullability, SmallInt, SqlReturn};
use std::sync::RwLock;

mod unit {
    use odbc_sys::SqlDataType;

    use super::*;
    // test unallocated_statement tests SQLColAttributeW when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_string_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        for desc in [
            Desc::BaseColumnName,
            Desc::BaseTableName,
            Desc::CatalogName,
            Desc::Label,
            Desc::LiteralPrefix,
            Desc::LiteralSuffix,
            Desc::Name,
            Desc::TableName,
            Desc::TypeName,
        ] {
            unsafe {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let buffer_length: SmallInt = 20;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLColAttributeW(
                        stmt_handle as *mut _,
                        0,
                        desc,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(
                    "[MongoDB][API] No resultset for statement".to_string(),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .errors
                            .read()
                            .unwrap()[0]
                    ),
                );
                let _ = Box::from_raw(char_buffer);
            }
        }
    }

    #[test]
    fn unallocated_statement_numeric_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        for desc in [
            Desc::AutoUniqueValue,
            Desc::CaseSensitive,
            Desc::Count,
            Desc::DisplaySize,
            Desc::FixedPrecScale,
            Desc::Length,
            Desc::Nullable,
            Desc::OctetLength,
            Desc::Precision,
            Desc::Scale,
            Desc::Searchable,
            Desc::Type,
            Desc::ConciseType,
            Desc::Unnamed,
            Desc::Updatable,
            Desc::Unsigned,
        ] {
            unsafe {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let buffer_length: SmallInt = 20;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLColAttributeW(
                        stmt_handle as *mut _,
                        0,
                        desc,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(
                    "[MongoDB][API] No resultset for statement".to_string(),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .errors
                            .read()
                            .unwrap()[0]
                    ),
                );
                let _ = Box::from_raw(char_buffer);
            }
        }
    }

    #[test]
    fn unallocated_statement_describe_col() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let name_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let name_buffer_length: SmallInt = 20;
            let out_name_length = &mut 10;
            let mut data_type = SqlDataType::UNKNOWN_TYPE;
            let col_size = &mut 42usize;
            let decimal_digits = &mut 42i16;
            let mut nullable = Nullability::NO_NULLS;
            // test string attributes
            assert_eq!(
                SqlReturn::ERROR,
                SQLDescribeColW(
                    stmt_handle as *mut _,
                    0,
                    name_buffer as *mut _,
                    name_buffer_length,
                    out_name_length,
                    &mut data_type,
                    col_size,
                    decimal_digits,
                    &mut nullable,
                )
            );
            assert_eq!(
                "[MongoDB][API] No resultset for statement".to_string(),
                format!(
                    "{}",
                    (*stmt_handle)
                        .as_statement()
                        .unwrap()
                        .errors
                        .read()
                        .unwrap()[0]
                ),
            );
            let _ = Box::from_raw(name_buffer);
        }
    }

    #[test]
    fn unallocated_statement_unsupported_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        for desc in [
            Desc::OctetLengthPtr,
            Desc::DatetimeIntervalCode,
            Desc::IndicatorPtr,
            Desc::DataPtr,
            Desc::AllocType,
            Desc::ArraySize,
            Desc::ArrayStatusPtr,
            Desc::BindOffsetPtr,
            Desc::BindType,
            Desc::DatetimeIntervalPrecision,
            Desc::MaximumScale,
            Desc::MinimumScale,
            Desc::NumPrecRadix,
            Desc::ParameterType,
            Desc::RowsProcessedPtr,
            Desc::RowVer,
        ] {
            unsafe {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let buffer_length: SmallInt = 20;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLColAttributeW(
                        stmt_handle as *mut _,
                        0,
                        desc,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(
                    "[MongoDB][API] No resultset for statement".to_string(),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .errors
                            .read()
                            .unwrap()[0]
                    ),
                );
                let _ = Box::from_raw(char_buffer);
            }
        }
    }

    #[test]
    fn test_index_out_of_bounds_describe() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
        unsafe {
            for col_index in [0, 30] {
                let name_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let name_buffer_length: SmallInt = 20;
                let out_name_length = &mut 10;
                let mut data_type = SqlDataType::UNKNOWN_TYPE;
                let col_size = &mut 42usize;
                let decimal_digits = &mut 42i16;
                let mut nullable = Nullability::NO_NULLS;
                // test string attributes
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLDescribeColW(
                        stmt_handle as *mut _,
                        col_index,
                        name_buffer as *mut _,
                        name_buffer_length,
                        out_name_length,
                        &mut data_type,
                        col_size,
                        decimal_digits,
                        &mut nullable,
                    )
                );
                assert_eq!(
                    format!(
                        "[MongoDB][API] The field index {} is out of bounds",
                        col_index,
                    ),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .errors
                            .read()
                            .unwrap()[0]
                    )
                );
                let _ = Box::from_raw(name_buffer);
            }
        }
    }

    #[test]
    fn test_index_out_of_bounds_attr() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
        for desc in [
            // string descriptor
            Desc::TypeName,
            // numeric descriptor
            Desc::Type,
        ] {
            unsafe {
                for col_index in [0, 30] {
                    let char_buffer: *mut std::ffi::c_void =
                        Box::into_raw(Box::new([0u8; 40])) as *mut _;
                    let buffer_length: SmallInt = 20;
                    let out_length = &mut 10;
                    let numeric_attr_ptr = &mut 10;
                    // test string attributes
                    assert_eq!(
                        SqlReturn::ERROR,
                        SQLColAttributeW(
                            mongo_handle as *mut _,
                            col_index,
                            desc,
                            char_buffer,
                            buffer_length,
                            out_length,
                            numeric_attr_ptr,
                        )
                    );
                    assert_eq!(
                        format!(
                            "[MongoDB][API] The field index {} is out of bounds",
                            col_index,
                        ),
                        format!(
                            "{}",
                            (*mongo_handle)
                                .as_statement()
                                .unwrap()
                                .errors
                                .read()
                                .unwrap()[0]
                        )
                    );
                    let _ = Box::from_raw(char_buffer);
                }
            }
        }
    }

    // check the fields column for all the string attributes
    #[test]
    fn test_string_field_attributes() {
        unsafe {
            let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
            stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
            let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
            let col_index = 3; //TABLE_NAME
            for (desc, expected) in [
                (Desc::BaseColumnName, ""),
                (Desc::BaseTableName, ""),
                (Desc::CatalogName, ""),
                (Desc::Label, "TABLE_NAME"),
                (Desc::LiteralPrefix, ""),
                (Desc::LiteralSuffix, ""),
                (Desc::Name, "TABLE_NAME"),
                (Desc::TableName, ""),
                (Desc::TypeName, "string"),
            ] {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let buffer_length: SmallInt = 20;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLColAttributeW(
                        mongo_handle as *mut _,
                        col_index,
                        desc,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(expected.len() as i16, *out_length);
                assert_eq!(
                    expected,
                    crate::api::data::input_wtext_to_string(
                        char_buffer as *const _,
                        *out_length as usize
                    )
                );
                let _ = Box::from_raw(char_buffer);
            }
        }
    }

    // check the fields column for all the numeric attributes
    #[test]
    fn test_numeric_field_attributes() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
        let col_index = 3; //TABLE_NAME
        for (desc, expected) in [
            (Desc::AutoUniqueValue, 0isize),
            (Desc::Unnamed, 0),
            (Desc::Updatable, 0),
            (Desc::Count, 18),
            (Desc::CaseSensitive, 1),
            (Desc::DisplaySize, 0),
            (Desc::FixedPrecScale, 0),
            (Desc::Length, 0),
            (Desc::Nullable, 0),
            (Desc::OctetLength, 0),
            (Desc::Precision, 0),
            (Desc::Scale, 0),
            (Desc::Searchable, 1),
            (Desc::Type, SqlDataType::VARCHAR.0 as isize),
            (Desc::ConciseType, SqlDataType::VARCHAR.0 as isize),
            (Desc::Unsigned, 0),
        ] {
            unsafe {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let buffer_length: SmallInt = 20;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLColAttributeW(
                        mongo_handle as *mut _,
                        col_index,
                        desc,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(expected, *numeric_attr_ptr);
                let _ = Box::from_raw(char_buffer);
            }
        }
    }

    // check the describe output
    #[test]
    fn test_describe_col() {
        unsafe {
            let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
            stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
            let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
            let col_index = 3; //TABLE_NAME
            let name_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let name_buffer_length: SmallInt = 20;
            let out_name_length = &mut 0;
            let mut data_type = SqlDataType::UNKNOWN_TYPE;
            let col_size = &mut 42usize;
            let decimal_digits = &mut 42i16;
            let mut nullable = Nullability::UNKNOWN;
            // test string attributes
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLDescribeColW(
                    mongo_handle as *mut _,
                    col_index,
                    name_buffer as *mut _,
                    name_buffer_length,
                    out_name_length,
                    &mut data_type,
                    col_size,
                    decimal_digits,
                    &mut nullable,
                )
            );
            // out_name_length should be 10
            assert_eq!(10, *out_name_length);
            // data_type should be VARCHAR
            assert_eq!(SqlDataType::VARCHAR, data_type);
            // col_size should be 0
            assert_eq!(0usize, *col_size);
            // decimal_digits should be 0
            assert_eq!(0i16, *decimal_digits);
            // nullable should stay as NO_NULLS
            assert_eq!(Nullability::NO_NULLS, nullable);
            // name_buffer should contain TABLE_NAME
            assert_eq!(
                "TABLE_NAME".to_string(),
                crate::api::data::input_wtext_to_string(
                    name_buffer as *const _,
                    *out_name_length as usize
                )
            );
            let _ = Box::from_raw(name_buffer);
        }
    }
}
