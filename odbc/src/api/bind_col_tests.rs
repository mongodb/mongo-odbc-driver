mod unit {
    use crate::{
        handles::definitions::{
            BoundColInfo, Connection, ConnectionState, Env, EnvState, MongoHandle, Statement,
            StatementState,
        },
        map, SQLBindCol, SQLFetch,
    };
    use bson::doc;
    use cstr::{input_text_to_string_w, WideChar};
    use definitions::{
        BindType, CDataType, Len, Nullability,
        RowStatus::{SQL_ROW_NOROW, SQL_ROW_SUCCESS},
        SmallInt, SqlReturn, ULen, USmallInt, WChar,
    };
    use mongo_odbc_core::{
        json_schema::{
            simplified::{Atomic, Schema},
            BsonTypeName,
        },
        mock_query::MongoQuery,
        MongoColMetadata, MongoStatement, TypeMode,
    };
    use std::collections::HashMap;
    use std::ptr::null_mut;

    #[test]
    fn test_binding_and_rebinding_column() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut create_mongo_query_for_bind_col_tests();

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let indicator: *mut Len = null_mut();

            // Test binding a new column
            let new_binding_buffer: *mut std::ffi::c_void =
                Box::into_raw(Box::new([0u8; 4])) as *mut _;

            // Assert that SQLBindCol is successful
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG as SmallInt,
                    new_binding_buffer,
                    4,
                    indicator
                )
            );

            // Assert that bound_cols has the correct value inside
            assert_eq!(
                Some(map! {
                    1 => BoundColInfo {
                        target_type: CDataType::SQL_C_SLONG as SmallInt,
                        target_buffer: new_binding_buffer,
                        buffer_length: 4,
                        length_or_indicator: indicator,
                    }
                }),
                *s.bound_cols.read().unwrap()
            );

            // Test rebinding a column
            let rebinding_buffer: *mut std::ffi::c_void =
                Box::into_raw(Box::new([0u8; 4])) as *mut _;

            // Assert that SQLBindCol is successful
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG as SmallInt,
                    rebinding_buffer,
                    4,
                    indicator
                )
            );

            // Assert that bound_cols has the correct value inside
            assert_eq!(
                Some(map! {
                    1 => BoundColInfo {
                        target_type: CDataType::SQL_C_SLONG as SmallInt,
                        target_buffer: rebinding_buffer,
                        buffer_length: 4,
                        length_or_indicator: indicator,
                    }
                }),
                *s.bound_cols.read().unwrap()
            );

            // free buffers
            let _ = Box::from_raw(new_binding_buffer as *mut WChar);
            let _ = Box::from_raw(rebinding_buffer as *mut WChar);
        }
    }

    #[test]
    fn test_unbinding_column() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // Set the bound_cols to non-None initially.
            *s.bound_cols.write().unwrap() = Some(map! {
                1 => BoundColInfo {
                    target_type: CDataType::SQL_C_SLONG as SmallInt,
                    target_buffer: null_mut(),
                    buffer_length: 1,
                    length_or_indicator: null_mut(),
                },
                2 => BoundColInfo {
                    target_type: CDataType::SQL_C_SLONG as SmallInt,
                    target_buffer: null_mut(),
                    buffer_length: 1,
                    length_or_indicator: null_mut(),
                }
            });

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut create_mongo_query_for_bind_col_tests();

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let indicator: *mut Len = null_mut();

            // Assert that SQLBindCol is successful
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG as SmallInt,
                    null_mut(), // Note that when TargetValuePtr is null, the driver unbinds the data buffer for the column specified by ColumnNumber
                    0,
                    indicator
                )
            );

            // Assert that bound_cols only has one mapping after SQLBindCol is called.
            assert_eq!(
                Some(map! {2 => BoundColInfo {
                    target_type: CDataType::SQL_C_SLONG as SmallInt,
                    target_buffer: null_mut(),
                    buffer_length: 1,
                    length_or_indicator: null_mut(),
                }}),
                *s.bound_cols.read().unwrap()
            )
        }
    }

    #[test]
    fn test_invalid_column_number() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut create_mongo_query_for_bind_col_tests();

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let indicator: *mut Len = null_mut();
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 4])) as *mut _;

            // Assert that SQLBindCol returns an error
            assert_eq!(
                SqlReturn::ERROR,
                SQLBindCol(
                    stmt as *mut _,
                    3,
                    CDataType::SQL_C_SLONG as SmallInt,
                    buffer,
                    4,
                    indicator
                )
            );

            // free buffer
            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_invalid_target_type() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut create_mongo_query_for_bind_col_tests();

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let indicator: *mut Len = null_mut();
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 4])) as *mut _;

            // Assert that SQLBindCol returns an error. The target_type is set to 500 which is an arbitrary, invalid target_type.
            assert_eq!(
                SqlReturn::ERROR,
                SQLBindCol(stmt as *mut _, 1, 500, buffer, 4, indicator)
            );

            // free buffer
            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_unsupported_ways_to_column_bind() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut create_mongo_query_for_bind_col_tests();

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let indicator: *mut Len = null_mut();
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 4])) as *mut _;

            // set all statement attributes to the correct values except for row_bind_offset_ptr.
            s.attributes.write().unwrap().row_bind_offset_ptr =
                Box::into_raw(Box::new(100)) as *mut ULen;
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Assert that SQLBindCol returns an error because row_bind_offset_ptr is not null.
            assert_eq!(
                SqlReturn::ERROR,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG as SmallInt,
                    buffer,
                    4,
                    indicator
                )
            );

            // Free memory and set row_bind_offset_ptr to null. Set row_bind_type to an invalid number.
            let _ = Box::from_raw(s.attributes.write().unwrap().row_bind_offset_ptr as *mut WChar);
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_bind_type = 10;

            // Assert that SQLBindCol returns an error because row_bind_type is not 0 (i.e., BindType::SQL_BIND_BY_COLUMN).
            assert_eq!(
                SqlReturn::ERROR,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG as SmallInt,
                    buffer,
                    4,
                    indicator
                )
            );

            // free buffer
            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_binding_arrays_when_rowset_size_evenly_divides_resultset_size() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set every value in the array to 0, so we know SQLFetch changed the values when we check later.
            // num_buffer needs to be 8 bytes since the rowset size is 2, and each column value is an i32 (4 bytes), so (2 rows)*(4 bytes) = 8 bytes.
            let num_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 8])) as *mut _;
            let num_indicator: *mut Len = Box::into_raw(Box::new([0isize; 2])) as *mut Len;

            // word_buffer needs to be 40 bytes since the rowset size is 2, and each column value is a string of 5 WideChars (4 bytes)
            // including the null termination character, so (2 rows)*(5 * 4 bytes) = 40 bytes.
            let word_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let word_indicator: *mut Len = Box::into_raw(Box::new([0isize; 2])) as *mut Len;

            // In this test, we assume that SQLBindCol has already been run and added columns to bind, so
            // I add column `1` and `2` to bound_cols.
            *s.bound_cols.write().unwrap() = create_column_bindings_for_num_and_word(
                num_buffer,
                num_indicator,
                word_buffer,
                word_indicator,
            );

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            // row_array_size is 2 meaning sqlFetch will fetch and handle the column bindings for 2 rows at a time.
            s.attributes.write().unwrap().row_array_size = 2;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // row_status_ptr needs 2 slots since the rowset size is 2.
            s.attributes.write().unwrap().row_status_ptr =
                Box::into_raw(Box::new([0u16; 2])) as *mut USmallInt;
            s.attributes.write().unwrap().rows_fetched_ptr =
                Box::into_raw(Box::new(0usize)) as *mut ULen;

            // create a mongo query with data that corresponds to the bound columns.
            let mock_query = create_mongo_query_for_bind_col_fetching_tests();

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query));

            // assert that SQLFetch is successful. We are fetching the first 2 rows in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the first 2 values from the result set were put in the bound buffer array correctly for column 1.
            assert_eq!(10, *(num_buffer as *mut i32));
            assert_eq!(20, *((num_buffer as ULen + 4) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));

            // assert that the first 2 values from the result set were put in the bound buffer array correctly for column 2.
            // input_text_to_string_w requires a `usize` value; however, I need a negative value to test that the null termination character was added,
            // So I input usize::MAX because it casts to -1 in the function.
            assert_eq!(
                "aaaa",
                input_text_to_string_w(word_buffer as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "bbbb",
                input_text_to_string_w((word_buffer as ULen + 20) as *const WideChar, usize::MAX)
            );

            // assert that the indicator has the correct values for column 2.
            assert_eq!(8, *word_indicator);
            assert_eq!(8, *((word_indicator as ULen + 8) as *mut Len));

            // assert that the rows_fetch_ptr has the correct value.
            assert_eq!(2, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_SUCCESS as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_SUCCESS as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );

            // assert that SQLFetch is successful. We are fetching the last 2 rows in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the last 2 values from the result set were put in the bound buffer array correctly for column 1.
            assert_eq!(30, *(num_buffer as *mut i32));
            assert_eq!(40, *((num_buffer as ULen + 4) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));

            // assert that the last 2 values from the result set were put in the bound buffer array correctly for column 2.
            assert_eq!(
                "cccc",
                input_text_to_string_w(word_buffer as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "dddd",
                input_text_to_string_w((word_buffer as ULen + 20) as *const WideChar, usize::MAX)
            );

            // assert that the indicator has the correct values for column 2.
            assert_eq!(8, *word_indicator);
            assert_eq!(8, *((word_indicator as ULen + 8) as *mut Len));

            // assert that the rows_fetch_ptr has the correct value.
            assert_eq!(2, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_SUCCESS as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_SUCCESS as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );

            // assert that another fetch returns NO_DATA
            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt as *mut _));

            // make sure rows_fetch_ptr is set to 0.
            assert_eq!(0, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_NOROW as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_NOROW as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );

            // free buffers
            let _ = Box::from_raw(num_buffer as *mut WChar);
            let _ = Box::from_raw(num_indicator as *mut WChar);

            let _ = Box::from_raw(word_buffer as *mut WChar);
            let _ = Box::from_raw(word_indicator as *mut WChar);

            let _ = Box::from_raw(s.attributes.write().unwrap().row_status_ptr as *mut WChar);
            let _ = Box::from_raw(s.attributes.write().unwrap().rows_fetched_ptr as *mut WChar);
        }
    }
    #[test]
    fn test_binding_arrays_when_rowset_size_doesnt_evenly_divide_resultset_size() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set every value in the array to 0, so we know SQLFetch changed the values when we check later.
            // num_buffer needs to be 12 bytes since the rowset size is 3, and each column value is an i32 (4 bytes), so (3 rows)*(4 bytes) = 12 bytes.
            let num_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 12])) as *mut _;
            let num_indicator: *mut Len = Box::into_raw(Box::new([0isize; 3])) as *mut Len;

            // word_buffer needs to be 60 bytes since the rowset size is 3, and each column value is a string of 5 WideChars (4 bytes)
            // including the null termination character, so (3 rows)*(5 * 4 bytes) = 60 bytes.
            let word_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 60])) as *mut _;
            let word_indicator: *mut Len = Box::into_raw(Box::new([0isize; 3])) as *mut Len;

            // In this test, we assume that SQLBindCol has already been run and added columns to bind, so
            // I add column `1` and `2` to bound_cols.
            *s.bound_cols.write().unwrap() = create_column_bindings_for_num_and_word(
                num_buffer,
                num_indicator,
                word_buffer,
                word_indicator,
            );

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            // row_array_size is 3 meaning sqlFetch will fetch and handle the column bindings for 3 rows at a time.
            s.attributes.write().unwrap().row_array_size = 3;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // row_status_ptr needs 3 slots since the rowset size is 3.
            s.attributes.write().unwrap().row_status_ptr =
                Box::into_raw(Box::new([0u16; 3])) as *mut USmallInt;
            s.attributes.write().unwrap().rows_fetched_ptr =
                Box::into_raw(Box::new(0usize)) as *mut ULen;

            // create a mongo query with data that corresponds to the bound columns.
            let mock_query = create_mongo_query_for_bind_col_fetching_tests();

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query));

            // assert that SQLFetch is successful. We are fetching the first 3 rows in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the first 3 values from the result set were put in the bound buffer array correctly for column 1.
            assert_eq!(10, *(num_buffer as *mut i32));
            assert_eq!(20, *((num_buffer as ULen + 4) as *mut i32));
            assert_eq!(30, *((num_buffer as ULen + 8) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));
            assert_eq!(4, *((num_indicator as ULen + 16) as *mut Len));

            // assert that the first 3 values from the result set were put in the bound buffer array correctly for column 2.
            // input_text_to_string_w requires a `usize` value; however, I need a negative value to test that the null termination character was added,
            // So I input usize::MAX because it casts to -1 in the function.
            assert_eq!(
                "aaaa",
                input_text_to_string_w(word_buffer as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "bbbb",
                input_text_to_string_w((word_buffer as ULen + 20) as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "cccc",
                input_text_to_string_w((word_buffer as ULen + 40) as *const WideChar, usize::MAX)
            );

            // assert that the indicator has the correct values for column 2.
            assert_eq!(8, *word_indicator);
            assert_eq!(8, *((word_indicator as ULen + 8) as *mut Len));
            assert_eq!(8, *((word_indicator as ULen + 16) as *mut Len));

            // assert that the rows_fetch_ptr has the correct value.
            assert_eq!(3, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_SUCCESS as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_SUCCESS as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );
            assert_eq!(
                SQL_ROW_SUCCESS as USmallInt,
                *((row_status_array as ULen + 4) as *mut USmallInt)
            );

            // assert that SQLFetch is successful. We are fetching the last row in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the last value from the result set was put in the bound buffer array correctly for column 1.
            assert_eq!(40, *(num_buffer as *mut i32));
            assert_eq!(20, *((num_buffer as ULen + 4) as *mut i32));
            assert_eq!(30, *((num_buffer as ULen + 8) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));
            assert_eq!(4, *((num_indicator as ULen + 16) as *mut Len));

            // assert that the last value from the result set was put in the bound buffer array correctly for column 2.
            assert_eq!(
                "dddd",
                input_text_to_string_w(word_buffer as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "bbbb",
                input_text_to_string_w((word_buffer as ULen + 20) as *const WideChar, usize::MAX)
            );
            assert_eq!(
                "cccc",
                input_text_to_string_w((word_buffer as ULen + 40) as *const WideChar, usize::MAX)
            );

            // assert that the indicator has the correct values for column 2.
            assert_eq!(8, *word_indicator);
            assert_eq!(8, *((word_indicator as ULen + 8) as *mut Len));
            assert_eq!(8, *((word_indicator as ULen + 16) as *mut Len));

            // assert that the rows_fetch_ptr has the correct value.
            assert_eq!(1, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_SUCCESS as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_NOROW as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );
            assert_eq!(
                SQL_ROW_NOROW as USmallInt,
                *((row_status_array as ULen + 4) as *mut USmallInt)
            );

            // assert that another fetch returns NO_DATA
            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt as *mut _));

            // make sure rows_fetch_ptr is set to 0.
            assert_eq!(0, *s.attributes.read().unwrap().rows_fetched_ptr);

            // check the row status array
            let row_status_array = s.attributes.read().unwrap().row_status_ptr;

            assert_eq!(SQL_ROW_NOROW as USmallInt, *row_status_array);
            assert_eq!(
                SQL_ROW_NOROW as USmallInt,
                *((row_status_array as ULen + 2) as *mut USmallInt)
            );
            assert_eq!(
                SQL_ROW_NOROW as USmallInt,
                *((row_status_array as ULen + 4) as *mut USmallInt)
            );

            // free buffers
            let _ = Box::from_raw(num_buffer as *mut WChar);
            let _ = Box::from_raw(num_indicator as *mut WChar);

            let _ = Box::from_raw(word_buffer as *mut WChar);
            let _ = Box::from_raw(word_indicator as *mut WChar);

            let _ = Box::from_raw(s.attributes.write().unwrap().row_status_ptr as *mut WChar);
            let _ = Box::from_raw(s.attributes.write().unwrap().rows_fetched_ptr as *mut WChar);
        }
    }

    #[test]
    fn test_binding_arrays_without_row_status_and_rows_fetched_ptrs() {
        // Set up MongoHandle
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Get Statement
            let s = (*stmt).as_statement().unwrap();

            // set every value in the array to 0, so we know SQLFetch changed the values when we check later.
            // num_buffer needs to be 8 bytes since the rowset size is 2, and each column value is an i32 (4 bytes), so (2 rows)*(4 bytes) = 8 bytes.
            let num_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 8])) as *mut _;
            let num_indicator: *mut Len = Box::into_raw(Box::new([0isize; 2])) as *mut Len;

            // In this test, we assume that SQLBindCol has already been run and added a column to bind, so
            // I add column "1" to bound_cols.
            *s.bound_cols.write().unwrap() = Some(map! {
                1 => BoundColInfo {
                    target_type: CDataType::SQL_C_SLONG as SmallInt,
                    target_buffer: num_buffer,
                    buffer_length: 4, // buffer_length is 4 because an i32 is 4 bytes; therefore, each buffer needs to be 4 bytes long.
                    length_or_indicator: num_indicator,
                },

            });

            // set all statement attributes to the correct values.
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            // row_array_size is 2 meaning sqlFetch will fetch and handle the column bindings for 2 rows at a time.
            s.attributes.write().unwrap().row_array_size = 2;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // make both of these attributes null to signify that they have not been set.
            s.attributes.write().unwrap().row_status_ptr = null_mut();
            s.attributes.write().unwrap().rows_fetched_ptr = null_mut();

            // create a mongo query with data that corresponds to the bound columns.
            let mock_query = create_mongo_query_for_bind_col_fetching_tests();

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query));

            // assert that SQLFetch is successful. We are fetching the first 2 rows in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the first 2 values from the result set were put in the bound buffer array correctly for column 1
            assert_eq!(10, *(num_buffer as *mut i32));
            assert_eq!(20, *((num_buffer as ULen + 4) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));

            // assert that SQLFetch is successful. We are fetching the next 4 rows in the result set.
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt as *mut _));

            // assert that the last 2 values from the result set were put in the bound buffer array correctly for column 1
            assert_eq!(30, *(num_buffer as *mut i32));
            assert_eq!(40, *((num_buffer as ULen + 4) as *mut i32));

            // assert that the indicator has the correct values for column 1. In this case, the indicator stores the length (in bytes) of the data available to return.
            // since each column value is an i32 (4 bytes), the num_indicator stores 4.
            assert_eq!(4, *num_indicator);
            assert_eq!(4, *((num_indicator as ULen + 8) as *mut Len));

            // assert that another fetch returns NO_DATA
            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt as *mut _));

            // free buffers
            let _ = Box::from_raw(num_buffer as *mut WChar);
            let _ = Box::from_raw(num_indicator as *mut WChar);
        }
    }

    fn create_mongo_query_for_bind_col_fetching_tests() -> MongoQuery {
        MongoQuery::new(
            vec![
                doc! {"test": {"num": 10, "word": "aaaa"}},
                doc! {"test": {"num": 20, "word": "bbbb"}},
                doc! {"test": {"num": 30, "word": "cccc"}},
                doc! {"test": {"num": 40, "word": "dddd"}},
            ],
            vec![
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "num".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "word".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple,
                ),
            ],
        )
    }

    fn create_mongo_query_for_bind_col_tests() -> MongoQuery {
        MongoQuery::new(
            vec![doc! {"x": 1}, doc! {"x": 2}],
            vec![MongoColMetadata::new(
                "",
                "".to_string(),
                "x".to_string(),
                Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                Nullability::SQL_NO_NULLS,
                TypeMode::Simple,
            )],
        )
    }

    fn create_column_bindings_for_num_and_word(
        num_buffer: *mut std::ffi::c_void,
        num_indicator: *mut Len,
        word_buffer: *mut std::ffi::c_void,
        word_indicator: *mut Len,
    ) -> Option<HashMap<USmallInt, BoundColInfo>> {
        Some(map! {
            1 => BoundColInfo {
                target_type: CDataType::SQL_C_SLONG as SmallInt,
                target_buffer: num_buffer,
                buffer_length: 4, // buffer_length is 4 because an i32 is 4 bytes; therefore, each buffer needs to be 4 bytes long.
                length_or_indicator: num_indicator,
            },
            2 => BoundColInfo {
                target_type: CDataType::SQL_C_WCHAR as SmallInt,
                target_buffer: word_buffer,
                buffer_length: 20, // buffer_length is 20 because each word is 20 bytes long including the null termination character (WideChar * 5).
                length_or_indicator: word_indicator,
            },

        })
    }
}
