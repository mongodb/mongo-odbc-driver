use crate::{
    handles::definitions::{MongoHandle, Statement, StatementState},
    map, SQLGetStmtAttrW, SQLSetStmtAttrW,
};
use definitions::{
    AsyncEnable, BindType, CursorScrollable, CursorSensitivity, CursorType, HStmt, Integer, NoScan,
    Pointer, RetrieveData, SqlBool, SqlReturn, StatementAttribute, ULen, USmallInt, UseBookmarks,
};
use std::{collections::BTreeMap, mem::size_of};

fn get_set_stmt_attr(
    handle: *mut MongoHandle,
    attribute: StatementAttribute,
    value_map: BTreeMap<i32, SqlReturn>,
    default_value: usize,
) {
    unsafe {
        let attr_buffer = Box::into_raw(Box::new(0_usize));
        let string_length_ptr = &mut 0;
        let attr = attribute as i32;

        // Test the statement attribute's default value
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetStmtAttrW(
                handle as *mut _,
                attr,
                attr_buffer as Pointer,
                0,
                string_length_ptr
            )
        );

        assert_eq!(default_value, { *attr_buffer });

        value_map
            .into_iter()
            .for_each(|(discriminant, expected_return)| {
                let value = discriminant as Pointer;
                assert_eq!(
                    expected_return,
                    SQLSetStmtAttrW(handle as HStmt, attr, value, 0)
                );
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetStmtAttrW(
                        handle as *mut _,
                        attr,
                        attr_buffer as Pointer,
                        0,
                        string_length_ptr
                    )
                );
                match expected_return {
                    SqlReturn::SUCCESS => {
                        assert_eq!(discriminant, { *attr_buffer } as i32)
                    }
                    _ => {
                        assert_eq!(default_value, { *attr_buffer })
                    }
                };
            });

        let _ = Box::from_raw(attr_buffer);
    }
}

fn get_set_ptr(
    handle: *mut MongoHandle,
    attribute: StatementAttribute,
    is_default_null: bool,
    is_set_attr_supported: bool,
    str_len: usize,
) {
    unsafe {
        let attr_buffer = Box::into_raw(Box::new(0_usize));
        let string_length_ptr = &mut 0;
        let attr = attribute as i32;

        // Test the statement attribute's default value
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetStmtAttrW(
                handle as *mut _,
                attr,
                attr_buffer as Pointer,
                0,
                string_length_ptr
            )
        );

        if is_default_null {
            assert!(
                (*attr_buffer as Pointer).is_null(),
                "Expected a null pointer for attribute {}, but got {}",
                attribute as u16,
                { *attr_buffer }
            );
        } else {
            assert!(
                !(*attr_buffer as Pointer).is_null(),
                "Expected an allocated pointer for attribute {}, but got a Null pointer",
                attribute as u16
            );
        }

        // attr_value can be any non-zero number.
        let attr_value = 10_usize;
        let (expected_value, expected_return) = match is_set_attr_supported {
            true => (attr_value, SqlReturn::SUCCESS),
            false => (*attr_buffer, SqlReturn::ERROR),
        };
        assert_eq!(
            expected_return,
            SQLSetStmtAttrW(handle as HStmt, attr, attr_value as Pointer, 0)
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLGetStmtAttrW(
                handle as *mut _,
                attr,
                attr_buffer as Pointer,
                0,
                string_length_ptr
            )
        );
        assert_eq!(expected_value, { *attr_buffer });
        assert_eq!(str_len as Integer, *string_length_ptr);

        let _ = Box::from_raw(attr_buffer);
    }
}

mod unit {
    use super::*;
    // test_supported_attributes tests SQLGetStmtAttr and SQLSetStmtAttr with every
    // supported statement attribute value.
    #[test]
    fn test_supported_attributes() {
        use crate::map;
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_APP_ROW_DESC,
            false,
            false,
            size_of::<Pointer>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_APP_PARAM_DESC,
            false,
            false,
            size_of::<Pointer>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_IMP_PARAM_DESC,
            false,
            false,
            size_of::<Pointer>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_IMP_ROW_DESC,
            false,
            false,
            size_of::<Pointer>(),
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_CURSOR_SCROLLABLE,
            map! {
                CursorScrollable::NonScrollable as i32 => SqlReturn::SUCCESS,
                CursorScrollable::Scrollable as i32 => SqlReturn::ERROR,
            },
            CursorScrollable::NonScrollable as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_CURSOR_SENSITIVITY,
            map! {
                CursorSensitivity::Insensitive as i32 => SqlReturn::SUCCESS,
                CursorSensitivity::Sensitive as i32 => SqlReturn::ERROR,
                CursorSensitivity::Unspecified as i32 => SqlReturn::ERROR
            },
            CursorSensitivity::Insensitive as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_CURSOR_TYPE,
            map! {
                CursorType::ForwardOnly as i32 => SqlReturn::SUCCESS,
                CursorType::Dynamic as i32 => SqlReturn::SUCCESS_WITH_INFO,
                CursorType::KeysetDriven as i32 => SqlReturn::SUCCESS_WITH_INFO,
                CursorType::Static as i32 => SqlReturn::SUCCESS_WITH_INFO,
            },
            CursorType::ForwardOnly as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_MAX_LENGTH,
            map! {
                10 => SqlReturn::ERROR, // Any number
            },
            0,
        );

        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_MAX_ROWS,
            map! {
                10 => SqlReturn::SUCCESS, // Any number
            },
            0,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_NOSCAN,
            map! {
                NoScan::Off as i32 => SqlReturn::SUCCESS,
                NoScan::On as i32 => SqlReturn::SUCCESS
            },
            NoScan::Off as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_QUERY_TIMEOUT,
            map! {
                10 => SqlReturn::SUCCESS, // Any number
            },
            0,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_RETRIEVE_DATA,
            map! {
                RetrieveData::Off as i32 => SqlReturn::SUCCESS,
                RetrieveData::On as i32 => SqlReturn::ERROR
            },
            RetrieveData::Off as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_BIND_TYPE,
            map! {
                BindType::BindByColumn as i32 => SqlReturn::SUCCESS,
                10 => SqlReturn::SUCCESS // Any number besides 0
            },
            BindType::BindByColumn as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_NUMBER,
            map! {
                10 => SqlReturn::SUCCESS // Any number
            },
            0,
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_STATUS_PTR,
            true,
            true,
            size_of::<*mut USmallInt>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROWS_FETCHED_PTR,
            true,
            true,
            size_of::<*mut ULen>(),
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE,
            map! {
                10 => SqlReturn::SUCCESS // Any number
            },
            1,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_USE_BOOKMARKS,
            map! {
                UseBookmarks::Off as i32 => SqlReturn::SUCCESS,
                UseBookmarks::Variable as i32 => SqlReturn::SUCCESS
            },
            UseBookmarks::Off as usize,
        );
    }

    // test_unsupported_attributes tests SQLGetStmtAttr and SQLSetStmtAttr with every
    // unsupported statement attribute value.
    #[test]
    fn test_unsupported_attributes() {
        use crate::map;
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));

        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ASYNC_ENABLE,
            map! {
                AsyncEnable::Off as i32 => SqlReturn::ERROR,
                AsyncEnable::On as i32 => SqlReturn::ERROR,
            },
            AsyncEnable::Off as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ENABLE_AUTO_IPD,
            map! {
                SqlBool::False as i32 => SqlReturn::ERROR,
                SqlBool::True as i32 => SqlReturn::ERROR,
            },
            SqlBool::False as usize,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_KEYSET_SIZE,
            map! {
                0 => SqlReturn::ERROR, // Any number
            },
            0,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_PARAM_BIND_TYPE,
            map! {
                0 => SqlReturn::ERROR, // Any number
            },
            0,
        );
        get_set_stmt_attr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_SIMULATE_CURSOR,
            map! {
                10 => SqlReturn::ERROR // Any number
            },
            0,
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ASYNC_STMT_EVENT,
            true,
            false,
            size_of::<Pointer>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_FETCH_BOOKMARK_PTR,
            true,
            false,
            size_of::<Pointer>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_PARAM_BIND_OFFSET_PTR,
            true,
            false,
            size_of::<*mut ULen>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_PARAM_OPERATION_PTR,
            true,
            false,
            size_of::<*mut USmallInt>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_PARAM_STATUS_PTR,
            true,
            false,
            size_of::<*mut USmallInt>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_PARAMS_PROCESSED_PTR,
            true,
            false,
            size_of::<*mut ULen>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_BIND_OFFSET_PTR,
            true,
            false,
            size_of::<*mut ULen>(),
        );
        get_set_ptr(
            stmt_handle,
            StatementAttribute::SQL_ATTR_ROW_OPERATION_PTR,
            true,
            false,
            size_of::<*mut USmallInt>(),
        );
    }
}
