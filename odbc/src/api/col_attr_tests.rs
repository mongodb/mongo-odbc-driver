use crate::{
    handles::definitions::{MongoHandle, Statement, StatementState},
    SQLColAttributeW,
};
use mongo_odbc_core::MongoFields;
use odbc_sys::{Desc, SmallInt, SqlReturn};
use std::sync::RwLock;

mod unit {
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
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        for desc in [
            // string descriptor
            Desc::TypeName,
            // numeric descriptor
            Desc::Type,
        ] {
            for col_index in [0, 1] {
                let char_buffer: *mut std::ffi::c_void = Vec::with_capacity(100).as_mut_ptr();
                let buffer_length: SmallInt = 100;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::ERROR,
                    SQLColAttributeW(
                        stmt_handle as *mut _,
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
            }
        }
    }

    // test_supported_attributes tests SQLColAttributeW with every
    // supported col attribute value.
    #[test]
    fn test_supported_attributes() {}

    // test_unsupported_attributes tests SQLColAttributeW with every
    // unsupported col attribute value.
    #[test]
    fn test_unsupported_attributes() {}
}
