use crate::{handles::definitions::*, SQLFetch, SQLGetDiagFieldW, SQLGetTypeInfoW};
use bson::Bson;
use definitions::{AttrOdbcVersion, DiagType, HandleType::Stmt, SqlDataType, SqlReturn};

const INVALID_SQL_TYPE: &str = "HY004\0";

mod unit {
    use super::*;
    use cstr::WideChar;
    use std::{ffi::c_void, mem::size_of};

    #[test]
    fn test_invalid_type_error() {
        // Test that a sql data type that is not defined in the enum yields the correct error
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));
        unsafe {
            assert_eq!(SqlReturn::ERROR, SQLGetTypeInfoW(stmt as *mut _, 100));
            // use SQLGetDiagField to retreive and assert correct error message
            let message_text = &mut [0; 6] as *mut _ as *mut c_void;
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetDiagFieldW(
                    Stmt,
                    stmt as *mut _,
                    1,
                    DiagType::SQL_DIAG_SQLSTATE as i16,
                    message_text,
                    6 * size_of::<WideChar>() as i16,
                    &mut 0
                )
            );
            assert_eq!(
                INVALID_SQL_TYPE,
                cstr::from_widechar_ref_lossy(&*(message_text as *const [WideChar; 6]))
            );
        }
    }

    #[test]
    fn test_invalid_cursor_state_error() {
        // checks for invalid cursor state when calling get_value before next
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let handle: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfoW(handle as *mut _, SqlDataType::SQL_INTEGER as i16)
            );
            let value = stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_value(1);
            assert!(value.is_err());
        }
    }

    #[test]
    fn test_odbc_2_returns_proper_date_type() {
        // Checks that when ODBC Version is set to 2, the date returned has the proper sql type, which shoudld be mapped in SQLGetTypeInfo
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        env.as_env().unwrap().attributes.write().unwrap().odbc_ver = AttrOdbcVersion::Odbc2;
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let handle: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfoW(handle as *mut _, SqlDataType::SQL_TYPE_TIMESTAMP as i16)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(handle as *mut _));
            let sql_type = stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_value(2)
                .unwrap();

            // EXT_TIMESTAMP is a code that was remapped in ODBC 3, but also stands for SQL_TIMESTAMP, the ODBC 2 type
            assert_eq!(
                sql_type,
                Some(Bson::Int32(SqlDataType::SQL_TIMESTAMP as i32))
            );
        }
    }

    #[test]
    fn test_odbc_3_returns_proper_date_type() {
        // Checks that when ODBC Version is set to 3, the date returned has the proper sql type
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        env.as_env().unwrap().attributes.write().unwrap().odbc_ver = AttrOdbcVersion::Odbc3_80;
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));
        let handle: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfoW(handle as *mut _, SqlDataType::SQL_TYPE_TIMESTAMP as i16)
            );
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(handle as *mut _));
            let sql_type = stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_value(2)
                .unwrap();

            // check the proper ODBC 3 sql type, SQL_TYPE_TIMESTAMP, is returned
            assert_eq!(
                sql_type,
                Some(Bson::Int32(SqlDataType::SQL_TYPE_TIMESTAMP as i32))
            );
        }
    }
}
