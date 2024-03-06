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
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            let s = (*stmt).as_statement().unwrap();

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

            let ind: *mut Len = null_mut();
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 4])) as *mut _;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(stmt as *mut _, 1, CDataType::SQL_C_SLONG, buffer, 4, ind)
            );

            // Assert that the bound_cols are None after SQLFreeStmt is called.
            assert_eq!(
                Some(map! {
                    1 => BoundColInfo {
                        target_type: CDataType::SQL_C_SLONG as SmallInt,
                        target_buffer: buffer,
                        buffer_length: 4,
                        length_or_indicator: ind,
                    }
                }),
                *s.bound_cols.read().unwrap()
            );

            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_unbind() {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Set the bound_cols to non-None initially.
            let s = (*stmt).as_statement().unwrap();
            *s.bound_cols.write().unwrap() = Some(map! {
                1 => BoundColInfo {
                    target_type: 1,
                    target_buffer: null_mut(),
                    buffer_length: 1,
                    length_or_indicator: null_mut(),
                }
            });

            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

            // Set the mongo_statement to have non-empty cursor initially.
            // Here, we create a MockQuery with nonsense dummy data since the
            // values themselves do not matter.
            let mock_query = &mut MongoQuery::new(
                vec![doc! {"x": "y"}, doc! {"x": "z"}],
                vec![MongoColMetadata::new(
                    "test_db",
                    "dn".to_string(),
                    "fn".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple,
                )],
            );

            // Must call next to set the `current` field.
            let _ = mock_query.next(None);

            // Set the mongo_statement
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            let ind: *mut Len = null_mut();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLBindCol(
                    stmt as *mut _,
                    1,
                    CDataType::SQL_C_NUMERIC,
                    null_mut(),
                    0,
                    ind
                )
            );

            // Assert that bound_cols is empty after SQLBindCol is called.
            assert_eq!(Some(map! {}), *s.bound_cols.read().unwrap())
        }
    }

    #[test]
    fn test_invalid_column_number() {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            let s = (*stmt).as_statement().unwrap();

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

            let ind: *mut Len = null_mut();
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 4])) as *mut _;
            assert_eq!(
                SqlReturn::ERROR,
                SQLBindCol(stmt as *mut _, 3, CDataType::SQL_C_SLONG, buffer, 4, ind)
            );

            let _ = Box::from_raw(buffer as *mut WChar);
        }
    }

    #[test]
    fn test_invalid_target_type() {
        //use SQLFetch instead because that's where the error comes from.
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            let s = (*stmt).as_statement().unwrap();

            s.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            s.attributes.write().unwrap().row_array_size = 1;
            s.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as usize;

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

            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt as *mut _));
        }
    }
}
