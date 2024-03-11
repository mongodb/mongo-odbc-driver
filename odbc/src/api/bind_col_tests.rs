mod unit {
    use crate::{
        handles::definitions::{
            BoundColInfo, Connection, ConnectionState, Env, EnvState, MongoHandle, Statement,
            StatementState,
        },
        map, SQLBindCol,
    };
    use bson::doc;
    use definitions::{BindType, CDataType, Len, Nullability, SmallInt, SqlReturn, ULen, WChar};
    use mongo_odbc_core::{
        json_schema::{
            simplified::{Atomic, Schema},
            BsonTypeName,
        },
        mock_query::MongoQuery,
        MongoColMetadata, MongoStatement, TypeMode,
    };
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
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                ],
            );

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
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                ],
            );

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
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                ],
            );

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
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                ],
            );

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
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                    MongoColMetadata::new(
                        "test_db",
                        "dn".to_string(),
                        "fn".to_string(),
                        Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        Nullability::SQL_NO_NULLS,
                        TypeMode::Simple,
                    ),
                ],
            );

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

            // Free memory and set row_bind_offset_ptr to null. Set row_array_size to an invalid number.
            let _ = Box::from_raw(s.attributes.write().unwrap().row_bind_offset_ptr as *mut WChar);
            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 100;

            // Assert that SQLBindCol returns an error because row_array_size is not 1.
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

            // set row_array_size to 1 and set row_bind_type to an invalid number.
            s.attributes.write().unwrap().row_array_size = 1;
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
}
