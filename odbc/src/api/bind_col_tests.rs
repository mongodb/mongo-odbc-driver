mod unit {
    use crate::{
        handles::definitions::{
            BoundColInfo, Connection, ConnectionState, Env, EnvState, MongoHandle, Statement,
            StatementState,
        },
        map,
    };
    use bson::doc;
    use definitions::{
        BindType, CDataType, Len, Nullability, SQLBindCol, SQLFetch, SmallInt, SqlReturn, WChar,
    };
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
    fn test_bind() {
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

            // Assert that SQLBindCol is successful
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_SLONG,
                    buffer,
                    4,
                    indicator
                )
            );

            // Assert that bound_cols has the correct value inside
            assert_eq!(
                Some(map! {
                    1 => BoundColInfo {
                        target_type: CDataType::SQL_C_SLONG as SmallInt,
                        target_buffer: buffer,
                        buffer_length: 4,
                        length_or_indicator: indicator,
                    }
                }),
                *s.bound_cols.read().unwrap()
            );

            // free buffer
            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_unbind() {
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
                    CDataType::SQL_C_SLONG,
                    null_mut(),
                    0,
                    indicator
                )
            );

            // Assert that bound_cols is an empty map after SQLBindCol is called.
            assert_eq!(Some(map! {}), *s.bound_cols.read().unwrap())
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
                    CDataType::SQL_C_SLONG,
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

            // Give bound_cols a column binding with an invalid target_type. The target_type check is in SQLFetch, so if
            // there is an attempt to create a column binding with an invalid target_type, it will be added to bind_cols,
            // and SQLBindCol will return SQLReturn::Success. Then, everything will operate as if nothing is wrong until
            // SQLFetch accesses bound_cols and checks for an invalid target_type. At this point, the error will occur.
            *s.bound_cols.write().unwrap() = Some(map! {
                1 => BoundColInfo {
                    target_type: 500,
                    target_buffer: null_mut(),
                    buffer_length: 1,
                    length_or_indicator: null_mut(),
                }
            });

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

            // Assert that SQLFetch returns an error because the target_type check is in SQLFetch instead of SQLBindCol
            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt as *mut _));
        }
    }
}
