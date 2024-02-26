mod unit {
    use crate::{
        handles::definitions::{
            BoundColInfo, Connection, ConnectionState, Env, EnvState, MongoHandle, Statement,
            StatementState,
        },
        map, SQLFreeStmt,
    };
    use bson::doc;
    use definitions::{FreeStmtOption, Nullability, SqlReturn};
    use mongo_odbc_core::{
        json_schema::{
            simplified::{Atomic, Schema},
            BsonTypeName,
        },
        mock_query::MongoQuery,
        Error, MongoColMetadata, MongoCollections, MongoStatement, TypeMode,
    };
    use std::ptr::null_mut;

    fn create_stmt_handle() -> *mut MongoHandle {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        stmt
    }

    #[test]
    fn test_free_stmt_invalid() {
        let stmt = create_stmt_handle();
        unsafe { assert_eq!(SqlReturn::ERROR, SQLFreeStmt(stmt as *mut _, 1)) }
    }

    #[test]
    fn test_free_stmt_reset_params() {
        let stmt = create_stmt_handle();
        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt as *mut _, FreeStmtOption::SQL_RESET_PARAMS as i16)
            )
        }
    }

    #[test]
    fn test_free_stmt_unbind() {
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
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt as *mut _, FreeStmtOption::SQL_UNBIND as i16)
            );

            // Assert that the bound_cols are None after SQLFreeStmt is called.
            assert_eq!(None, *s.bound_cols.read().unwrap())
        }
    }

    #[test]
    fn test_free_stmt_close() {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
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
            let s = (*stmt).as_statement().unwrap();
            *s.mongo_statement.write().unwrap() = Some(Box::new(mock_query.clone()));

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt as *mut _, FreeStmtOption::SQL_CLOSE as i16)
            );

            // Assert that the mongo_statement closed the cursor (no resultset, no current).
            match s
                .mongo_statement
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_value(1)
            {
                // we expect this error since there is no `current` set after SQLFreeStmt
                Err(Error::InvalidCursorState) => {}
                _ => panic!("cursor not closed -- able to call get_value()"),
            }

            match s
                .mongo_statement
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .next(None)
            {
                // we expect false since there should be no data to iterate after SQLFreeStmt
                Ok((false, _)) => {}
                _ => panic!("cursor not closed -- able to call next()"),
            }
        }
    }

    #[test]
    fn test_free_stmt_close_non_query() {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));

        unsafe {
            // Using a MongoCollections mongo_statement means there is no additional data
            // to set or test.
            let s = (*stmt).as_statement().unwrap();
            *s.mongo_statement.write().unwrap() = Some(Box::new(MongoCollections::empty()));

            assert_eq!(
                SqlReturn::SUCCESS,
                SQLFreeStmt(stmt as *mut _, FreeStmtOption::SQL_CLOSE as i16)
            );
        }
    }
}
