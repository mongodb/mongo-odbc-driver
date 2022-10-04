use crate::{
    handles::definitions::{MongoHandle, Statement, StatementState},
    SQLColAttributeW,
};
use mongo_odbc_core::MongoFields;
use odbc_sys::{Desc, SmallInt, SqlReturn};
use std::sync::RwLock;

mod unit {
    use odbc_sys::SqlDataType;

    use super::*;
    // test unallocated_statement tests SQLColAttributeW when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_string_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

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
            let char_buffer: *mut std::ffi::c_void = Vec::with_capacity(100).as_mut_ptr();
            let buffer_length: SmallInt = 100;
            let out_length = &mut 10;
            let numeric_attr_ptr = &mut 10;
            // test string attributes
            assert_eq!(
                SqlReturn::SUCCESS,
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
            // out_length was 10 should get changed to 0, denoting an empty output
            assert_eq!(0, *out_length);
            // numeric_attr_ptr should still be 10 since no numeric value was requested.
            assert_eq!(10, *numeric_attr_ptr);
        }
    }

    #[test]
    fn unallocated_statement_numeric_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

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
            let char_buffer: *mut std::ffi::c_void = Vec::with_capacity(100).as_mut_ptr();
            let buffer_length: SmallInt = 100;
            let out_length = &mut 10;
            let numeric_attr_ptr = &mut 10;
            // test string attributes
            assert_eq!(
                SqlReturn::SUCCESS,
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
            // out_length was 10 should stay 10, because a numeric attribute was selected
            assert_eq!(10, *out_length);
            // numeric_attr_ptr should change to 0 since a numeric attribute was requested.
            assert_eq!(0, *numeric_attr_ptr);
        }
    }

    #[test]
    fn unallocated_statement_unsupported_attr() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

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
            let char_buffer: *mut std::ffi::c_void = Vec::with_capacity(100).as_mut_ptr();
            let buffer_length: SmallInt = 100;
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
            // out_length should still be 10 since no string value was requested.
            assert_eq!(10, *out_length);
            // numeric_attr_ptr should still be 10 since no numeric value was requested.
            assert_eq!(10, *numeric_attr_ptr);
        }
    }

    #[test]
    fn test_index_out_of_bounds() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new(MongoFields::empty()));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        let mut i = 0;
        for desc in [
            // string descriptor
            Desc::TypeName,
            // numeric descriptor
            Desc::Type,
        ] {
            for col_index in [0, 30] {
                let char_buffer: *mut std::ffi::c_void = Vec::with_capacity(100).as_mut_ptr();
                let buffer_length: SmallInt = 100;
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
                // out_length should still be 10 since no string value was requested.
                assert_eq!(10, *out_length);
                // numeric_attr_ptr should still be 10 since no numeric value was requested.
                assert_eq!(10, *numeric_attr_ptr);
                unsafe {
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
                                .read()
                                .unwrap()
                                .errors[i]
                        )
                    );
                }
                i += 1;
            }
        }
    }

    // check the fields column for all the string attributes
    #[test]
    fn test_string_field_attributes() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new(MongoFields::empty()));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
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
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
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
                crate::api::functions::util::input_wtext_to_string(
                    char_buffer as *const _,
                    *out_length as usize
                )
            );
        }
    }

    // check the fields column for all the numeric attributes
    #[test]
    fn test_numeric_field_attributes() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new(MongoFields::empty()));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
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
            (Desc::Nullable, 1),
            (Desc::OctetLength, 0),
            (Desc::Precision, 0),
            (Desc::Scale, 0),
            (Desc::Searchable, 1),
            (Desc::Type, SqlDataType::VARCHAR.0 as isize),
            (Desc::ConciseType, SqlDataType::VARCHAR.0 as isize),
            (Desc::Unsigned, 0),
        ] {
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
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
        }
    }
}
