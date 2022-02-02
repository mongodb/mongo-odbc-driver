use crate::{
    api::definitions::*,
    handles::definitions::{MongoHandle, Statement, StatementState},
    map, SQLGetStmtAttrW, SQLSetStmtAttrW,
};
use odbc_sys::{HStmt, Integer, Pointer, SqlReturn, StatementAttribute, ULen, USmallInt};
use std::{collections::BTreeMap, mem::size_of, sync::RwLock};

fn get_set_stmt_attr(
    handle: *mut MongoHandle,
    attribute: StatementAttribute,
    value_map: BTreeMap<i32, SqlReturn>,
    default_value: i32,
) {
    let attr_buffer = Box::into_raw(Box::new(0));
    let string_length_ptr = &mut 0;

    // Test the statement attribute's default value
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetStmtAttrW(
            handle as *mut _,
            attribute,
            attr_buffer as Pointer,
            0,
            string_length_ptr
        )
    );

    assert_eq!(default_value, unsafe { *(attr_buffer as *const _) } as i32);

    value_map
        .into_iter()
        .for_each(|(discriminant, expected_return)| {
            let value = discriminant as Pointer;
            assert_eq!(
                expected_return,
                SQLSetStmtAttrW(handle as HStmt, attribute, value, 0)
            );
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetStmtAttrW(
                    handle as *mut _,
                    attribute,
                    attr_buffer as Pointer,
                    0,
                    string_length_ptr
                )
            );
            match expected_return {
                SqlReturn::SUCCESS => {
                    assert_eq!(discriminant, unsafe { *(attr_buffer as *const _) } as i32)
                }
                _ => {
                    assert_eq!(default_value, unsafe { *(attr_buffer as *const _) } as i32)
                }
            };
        });

    unsafe { Box::from_raw(attr_buffer) };
}

fn get_set_ptr(
    handle: *mut MongoHandle,
    attribute: StatementAttribute,
    attr_value: i32,
    is_attr_supported: bool,
    str_len: usize,
) {
    let attr_buffer = Box::into_raw(Box::new(0));
    let string_length_ptr = &mut 0;

    // Test the statement attribute's default value
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetStmtAttrW(
            handle as *mut _,
            attribute,
            attr_buffer as Pointer,
            0,
            string_length_ptr
        )
    );

    assert_eq!(0, unsafe { *attr_buffer });

    let expected_return = match is_attr_supported {
        true => SqlReturn::SUCCESS,
        false => SqlReturn::ERROR,
    };
    assert_eq!(
        expected_return,
        SQLSetStmtAttrW(handle as HStmt, attribute, attr_value as Pointer, 0)
    );
    assert_eq!(
        SqlReturn::SUCCESS,
        SQLGetStmtAttrW(
            handle as *mut _,
            attribute,
            attr_buffer as Pointer,
            0,
            string_length_ptr
        )
    );
    assert_eq!(attr_value, unsafe { *attr_buffer });
    assert_eq!(str_len as Integer, *string_length_ptr);

    unsafe { Box::from_raw(attr_buffer) };
}

// test_supported_attributes tests SQLGetStmtAttr and SQLSetStmtAttr with every
// supported statement attribute value.
#[test]
fn test_supported_attributes() {
    use crate::map;
    let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
        std::ptr::null_mut(),
        StatementState::Allocated,
    )));

    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::CursorScrollable,
        map! {
            CursorScrollable::NonScrollable as i32 => SqlReturn::SUCCESS,
            CursorScrollable::Scrollable as i32 => SqlReturn::ERROR,
        },
        CursorScrollable::NonScrollable as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::CursorSensitivity,
        map! {
            CursorSensitivity::Insensitive as i32 => SqlReturn::SUCCESS,
            CursorSensitivity::Sensitive as i32 => SqlReturn::ERROR,
            CursorSensitivity::Unspecified as i32 => SqlReturn::ERROR
        },
        CursorSensitivity::Insensitive as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::CursorType,
        map! {
            CursorType::ForwardOnly as i32 => SqlReturn::SUCCESS,
            CursorType::Dynamic as i32 => SqlReturn::SUCCESS_WITH_INFO,
            CursorType::KeysetDriven as i32 => SqlReturn::SUCCESS_WITH_INFO,
            CursorType::Static as i32 => SqlReturn::SUCCESS_WITH_INFO,
        },
        CursorType::ForwardOnly as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::MaxLength,
        map! {
            10 => SqlReturn::ERROR, // Any number
        },
        0,
    );

    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::MaxRows,
        map! {
            10 => SqlReturn::SUCCESS, // Any number
        },
        0,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::NoScan,
        map! {
            NoScan::Off as i32 => SqlReturn::SUCCESS,
            NoScan::On as i32 => SqlReturn::SUCCESS
        },
        NoScan::Off as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::QueryTimeout,
        map! {
            10 => SqlReturn::SUCCESS, // Any number
        },
        0,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::RetrieveData,
        map! {
            RetrieveData::Off as i32 => SqlReturn::SUCCESS,
            RetrieveData::On as i32 => SqlReturn::ERROR
        },
        RetrieveData::Off as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::RowBindType,
        map! {
            BindType::BindByColumn as i32 => SqlReturn::SUCCESS,
            10 => SqlReturn::SUCCESS // Any number besides 0
        },
        BindType::BindByColumn as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::RowNumber,
        map! {
            10 => SqlReturn::SUCCESS // Any number
        },
        0,
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::RowStatusPtr,
        10,
        true,
        size_of::<*mut USmallInt>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::RowsFetchedPtr,
        10,
        true,
        size_of::<*mut ULen>(),
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::RowArraySize,
        map! {
            10 => SqlReturn::SUCCESS // Any number
        },
        1,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::UseBookmarks,
        map! {
            UseBookmarks::Off as i32 => SqlReturn::SUCCESS,
            UseBookmarks::Variable as i32 => SqlReturn::SUCCESS
        },
        UseBookmarks::Off as i32,
    );
}

// test_unsupported_attributes tests SQLGetStmtAttr and SQLSetStmtAttr with every
// unsupported statement attribute value.
#[test]
fn test_unsupported_attributes() {
    use crate::map;
    let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
        std::ptr::null_mut(),
        StatementState::Allocated,
    )));

    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::AsyncEnable,
        map! {
            AsyncEnable::Off as i32 => SqlReturn::ERROR,
            AsyncEnable::On as i32 => SqlReturn::ERROR,
        },
        AsyncEnable::Off as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::EnableAutoIpd,
        map! {
            SqlBool::False as i32 => SqlReturn::ERROR,
            SqlBool::True as i32 => SqlReturn::ERROR,
        },
        SqlBool::False as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::KeysetSize,
        map! {
            0 => SqlReturn::ERROR, // Any number
        },
        0,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::ParamBindType,
        map! {
            0 as i32 => SqlReturn::ERROR, // Any number
        },
        0 as i32,
    );
    get_set_stmt_attr(
        stmt_handle,
        StatementAttribute::SimulateCursor,
        map! {
            10 => SqlReturn::ERROR // Any number
        },
        0,
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::AsyncStmtEvent,
        0,
        false,
        size_of::<Pointer>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::FetchBookmarkPtr,
        0,
        false,
        size_of::<Pointer>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::ParamBindOffsetPtr,
        0,
        false,
        size_of::<*mut ULen>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::ParamOpterationPtr,
        0,
        false,
        size_of::<*mut USmallInt>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::ParamStatusPtr,
        0,
        false,
        size_of::<*mut USmallInt>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::ParamsProcessedPtr,
        0,
        false,
        size_of::<*mut ULen>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::RowBindOffsetPtr,
        0,
        false,
        size_of::<*mut ULen>(),
    );
    get_set_ptr(
        stmt_handle,
        StatementAttribute::RowOperationPtr,
        0,
        false,
        size_of::<*mut USmallInt>(),
    );
}
