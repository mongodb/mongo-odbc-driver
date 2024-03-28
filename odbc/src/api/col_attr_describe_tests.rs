use crate::{
    handles::definitions::{MongoHandle, Statement, StatementState},
    SQLColAttributeW, SQLDescribeColW,
};
use definitions::{Desc, Nullability, SmallInt, SqlReturn, WChar};
use mongo_odbc_core::{MongoFields, SQL_SEARCHABLE};
use std::sync::RwLock;

mod unit {
    use definitions::SqlDataType;

    use crate::handles::definitions::{Connection, ConnectionState, Env, EnvState};

    use super::*;
    // test unallocated_statement tests SQLColAttributeW when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_string_attr() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);

        for desc in [
            Desc::SQL_DESC_BASE_COLUMN_NAME,
            Desc::SQL_DESC_BASE_TABLE_NAME,
            Desc::SQL_DESC_CATALOG_NAME,
            Desc::SQL_DESC_LABEL,
            Desc::SQL_DESC_LITERAL_PREFIX,
            Desc::SQL_DESC_LITERAL_SUFFIX,
            Desc::SQL_DESC_NAME,
            Desc::SQL_DESC_TABLE_NAME,
            Desc::SQL_DESC_TYPE_NAME,
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
                        desc as u16,
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
                let _ = Box::from_raw(char_buffer as *mut WChar);
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn unallocated_statement_numeric_attr() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);

        for desc in [
            Desc::SQL_DESC_AUTO_UNIQUE_VALUE,
            Desc::SQL_DESC_CASE_SENSITIVE,
            Desc::SQL_DESC_COUNT,
            Desc::SQL_DESC_DISPLAY_SIZE,
            Desc::SQL_DESC_FIXED_PREC_SCALE,
            Desc::SQL_DESC_LENGTH,
            Desc::SQL_DESC_NULLABLE,
            Desc::SQL_DESC_OCTET_LENGTH,
            Desc::SQL_DESC_PRECISION,
            Desc::SQL_DESC_SCALE,
            Desc::SQL_DESC_SEARCHABLE,
            Desc::SQL_DESC_TYPE,
            Desc::SQL_DESC_CONCISE_TYPE,
            Desc::SQL_DESC_UNNAMED,
            Desc::SQL_DESC_UPDATABLE,
            Desc::SQL_DESC_UNSIGNED,
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
                        desc as u16,
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
                let _ = Box::from_raw(char_buffer as *mut WChar);
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn unallocated_statement_describe_col() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);

        unsafe {
            let name_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let name_buffer_length: SmallInt = 20;
            let out_name_length = &mut 10;
            let mut data_type = SqlDataType::SQL_UNKNOWN_TYPE;
            let col_size = &mut 42usize;
            let decimal_digits = &mut 42i16;
            let mut nullable = Nullability::SQL_NO_NULLS as i16;
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
            let _ = Box::from_raw(name_buffer as *mut WChar);
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn unallocated_statement_unsupported_attr() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);

        for desc in [
            Desc::SQL_DESC_OCTET_LENGTH_PTR,
            Desc::SQL_DESC_DATETIME_INTERVAL_CODE,
            Desc::SQL_DESC_INDICATOR_PTR,
            Desc::SQL_DESC_DATA_PTR,
            Desc::SQL_DESC_ALLOC_TYPE,
            Desc::SQL_DESC_ARRAY_SIZE,
            Desc::SQL_DESC_ARRAY_STATUS_PTR,
            Desc::SQL_DESC_BIND_OFFSET_PTR,
            Desc::SQL_DESC_BIND_TYPE,
            Desc::SQL_DESC_DATETIME_INTERVAL_PRECISION,
            Desc::SQL_DESC_MAXIMUM_SCALE,
            Desc::SQL_DESC_MINIMUM_SCALE,
            Desc::SQL_DESC_NUM_PREC_RADIX,
            Desc::SQL_DESC_PARAMETER_TYPE,
            Desc::SQL_DESC_ROWS_PROCESSED_PTR,
            Desc::SQL_DESC_ROWVER,
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
                        desc as u16,
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
                let _ = Box::from_raw(char_buffer as *mut WChar);
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn test_index_out_of_bounds_describe() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);

        unsafe {
            for col_index in [0, 30] {
                let name_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 40])) as *mut _;
                let name_buffer_length: SmallInt = 20;
                let out_name_length = &mut 10;
                let mut data_type = SqlDataType::SQL_UNKNOWN_TYPE;
                let col_size = &mut 42usize;
                let decimal_digits = &mut 42i16;
                let mut nullable = Nullability::SQL_NO_NULLS as i16;
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
                    format!("[MongoDB][API] The column index {col_index} is out of bounds",),
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
                let _ = Box::from_raw(name_buffer as *mut WChar);
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn test_index_out_of_bounds_attr() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
        for desc in [
            // string descriptor
            Desc::SQL_DESC_TYPE_NAME,
            // numeric descriptor
            Desc::SQL_DESC_TYPE,
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
                            desc as u16,
                            char_buffer,
                            buffer_length,
                            out_length,
                            numeric_attr_ptr,
                        )
                    );
                    assert_eq!(
                        format!("[MongoDB][API] The field index {col_index} is out of bounds",),
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
                    let _ = Box::from_raw(char_buffer as *mut WChar);
                }
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    // check the fields column for all the string attributes
    #[test]
    fn test_string_field_attributes() {
        unsafe {
            let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
                EnvState::ConnectionAllocated,
            ))));
            let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
                env as *mut _,
                ConnectionState::Connected,
            ))));

            let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);

            stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
            let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
            let col_index = 3; //TABLE_NAME
            for (desc, expected) in [
                (Desc::SQL_DESC_BASE_COLUMN_NAME, ""),
                (Desc::SQL_DESC_BASE_TABLE_NAME, ""),
                (Desc::SQL_DESC_CATALOG_NAME, ""),
                (Desc::SQL_DESC_LABEL, "TABLE_NAME"),
                (Desc::SQL_DESC_LITERAL_PREFIX, "'"),
                (Desc::SQL_DESC_LITERAL_SUFFIX, "'"),
                (Desc::SQL_DESC_NAME, "TABLE_NAME"),
                (Desc::SQL_DESC_TABLE_NAME, ""),
                (Desc::SQL_DESC_TYPE_NAME, "string"),
            ] {
                let char_buffer: *mut std::ffi::c_void =
                    Box::into_raw(Box::new([0u8; 200])) as *mut _;
                let buffer_length: SmallInt = 200;
                let out_length = &mut 10;
                let numeric_attr_ptr = &mut 10;
                // test string attributes
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLColAttributeW(
                        mongo_handle as *mut _,
                        col_index,
                        desc as u16,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    )
                );
                assert_eq!(
                    (std::mem::size_of::<cstr::WideChar>() * expected.len()) as i16,
                    *out_length
                );
                assert_eq!(
                    expected,
                    cstr::input_text_to_string_w(char_buffer as *const _, expected.len(),)
                );
                let _ = Box::from_raw(char_buffer as *mut WChar);
            }
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    // check the fields column for all the numeric attributes
    #[test]
    fn test_numeric_field_attributes() {
        let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
            EnvState::ConnectionAllocated,
        ))));
        let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
            env as *mut _,
            ConnectionState::Connected,
        ))));

        let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
        stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
        let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
        let col_index = 3; //TABLE_NAME
        for (desc, expected) in [
            (Desc::SQL_DESC_AUTO_UNIQUE_VALUE, 0isize),
            (Desc::SQL_DESC_UNNAMED, 0),
            (Desc::SQL_DESC_UPDATABLE, 0),
            (Desc::SQL_DESC_COUNT, 18),
            (Desc::SQL_DESC_CASE_SENSITIVE, 1),
            (Desc::SQL_DESC_DISPLAY_SIZE, 0),
            (Desc::SQL_DESC_FIXED_PREC_SCALE, 0),
            (Desc::SQL_DESC_LENGTH, 0),
            (Desc::SQL_DESC_NULLABLE, 0),
            (Desc::SQL_DESC_OCTET_LENGTH, 0),
            (Desc::SQL_DESC_PRECISION, 0),
            (Desc::SQL_DESC_SCALE, 0),
            (Desc::SQL_DESC_SEARCHABLE, SQL_SEARCHABLE as isize),
            (Desc::SQL_DESC_TYPE, SqlDataType::SQL_WVARCHAR as isize),
            (
                Desc::SQL_DESC_CONCISE_TYPE,
                SqlDataType::SQL_WVARCHAR as isize,
            ),
            (Desc::SQL_DESC_UNSIGNED, 1),
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
                        desc as u16,
                        char_buffer,
                        buffer_length,
                        out_length,
                        numeric_attr_ptr,
                    ),
                    "expected success but got failure"
                );
                assert_eq!(
                    expected, *numeric_attr_ptr,
                    "expected {} but got {}",
                    expected, *numeric_attr_ptr
                );
                let _ = Box::from_raw(char_buffer as *mut WChar);
            }
        }
        unsafe {
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    // verify that given a column attribute that doesn't match any enum value, we return an informative error
    #[test]
    fn test_invalid_col_attribute() {
        unsafe {
            let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
                EnvState::ConnectionAllocated,
            ))));
            let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
                env as *mut _,
                ConnectionState::Connected,
            ))));

            let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
            stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
            let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);

            assert_eq!(
                SqlReturn::ERROR,
                SQLColAttributeW(
                    mongo_handle as *mut _,
                    0,
                    4, // not a valid field attribute
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
            );

            let errors = (*mongo_handle)
                .as_statement()
                .unwrap()
                .errors
                .read()
                .unwrap();
            assert_eq!(errors.len(), 1);
            assert_eq!(
                "[MongoDB][API] Invalid field descriptor value 4".to_string(),
                format!("{}", errors.first().unwrap()),
            );

            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    // check the describe output
    #[test]
    fn test_describe_col() {
        unsafe {
            let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
                EnvState::ConnectionAllocated,
            ))));
            let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
                env as *mut _,
                ConnectionState::Connected,
            ))));

            let mut stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);

            stmt.mongo_statement = RwLock::new(Some(Box::new(MongoFields::empty())));
            let mongo_handle: *mut _ = &mut MongoHandle::Statement(stmt);
            let col_index = 3; //TABLE_NAME
            let name_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let name_buffer_length: SmallInt = 20;
            let out_name_length = &mut 0;
            let mut data_type = SqlDataType::SQL_UNKNOWN_TYPE;
            let col_size = &mut 42usize;
            let decimal_digits = &mut 42i16;
            let mut nullable = Nullability::SQL_NULLABLE_UNKNOWN as i16;
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
            assert_eq!(SqlDataType::SQL_WVARCHAR, data_type);
            // col_size should be 0
            assert_eq!(0usize, *col_size);
            // decimal_digits should be 0
            assert_eq!(0i16, *decimal_digits);
            // nullable should stay as NO_NULLS
            assert_eq!(Nullability::SQL_NO_NULLS as i16, nullable);
            // name_buffer should contain TABLE_NAME
            assert_eq!(
                "TABLE_NAME".to_string(),
                cstr::input_text_to_string_w(name_buffer as *const _, *out_name_length as usize)
            );
            let _ = Box::from_raw(name_buffer as *mut WChar);
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }
}
