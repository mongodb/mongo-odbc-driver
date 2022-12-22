use crate::{handles::definitions::*, SQLGetTypeInfo};
use bson::Bson;
use odbc_sys::{SqlDataType, SqlReturn};

mod unit {
    use super::*;
    use mongo_odbc_core::Error;

    fn validate_result_set(data_type: SqlDataType, expectations: Vec<&str>) {
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let get_next = |stmt: &Statement| -> Result<bool, Error> {
                stmt.mongo_statement
                    .write()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .next(None)
            };
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(handle as *mut _, data_type)
            );
            let stmt = (*handle).as_statement().unwrap();
            // for each expectation, check that calling next succeeds and we get the expected value
            expectations.iter().for_each(|name| {
                let result = get_next(stmt);
                assert_eq!(true, result.unwrap());
                assert_eq!(
                    Bson::String((*name).to_string()),
                    stmt.mongo_statement
                        .write()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .get_value(1)
                        .unwrap()
                        .unwrap()
                )
            });

            // check there are no additional results in the set that were not expected
            let result = get_next(stmt);
            assert_eq!(false, result.unwrap());
        }
    }

    #[test]
    fn test_all_types() {
        let expectations = vec![
            "long",
            "binData",
            "array",
            "bson",
            "dbPointer",
            "decimal",
            "javascript",
            "javascriptWithScope",
            "maxKey",
            "minKey",
            "null",
            "object",
            "objectId",
            "symbol",
            "timestamp",
            "undefined",
            "int",
            "bool",
            "double",
            "string",
            "date",
        ];
        validate_result_set(SqlDataType(0), expectations);
    }

    #[test]
    fn test_specific_type_one_response() {
        let expectations = vec!["int"];
        validate_result_set(SqlDataType(4), expectations);
    }

    #[test]
    fn test_invalid_type() {
        // note: with an empty list of expectations, this will just check that calling next fails
        validate_result_set(SqlDataType(95), vec![])
    }

    #[test]
    fn test_invalid_cursor_state_error() {
        // checks for invalid cursor state when calling get_value before next
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(handle as *mut _, SqlDataType(4))
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
    fn test_get_all_values() {
        let handle: *mut _ = &mut MongoHandle::Statement(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        ));
        unsafe {
            let stmt = (*handle).as_statement().unwrap();
            assert_eq!(
                SqlReturn::SUCCESS,
                SQLGetTypeInfo(handle as *mut _, SqlDataType(4))
            );
            let result = stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .next(None);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
            // test each of the values come out properly for a given type
            let values: Vec<Bson> = vec![
                Bson::String("int".to_string()),
                Bson::Int32(4),
                Bson::Int32(10),
                Bson::Null,
                Bson::Null,
                Bson::Null,
                Bson::Boolean(false),
                Bson::Null,
                Bson::Int32(3),
                Bson::Int32(0),
                Bson::Boolean(true),
                Bson::Boolean(false),
                Bson::String("int".to_string()),
                Bson::Int32(0),
                Bson::Int32(0),
                Bson::Int32(4),
                Bson::Null,
                Bson::Int32(10),
                Bson::Null,
            ];
            for (col_index, value) in values.iter().enumerate() {
                assert_eq!(
                    *value,
                    stmt.mongo_statement
                        .write()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .get_value((col_index + 1) as u16)
                        .unwrap()
                        .unwrap()
                )
            }
            // test an out of bounds column index fails
            assert!(stmt
                .mongo_statement
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .get_value(20)
                .is_err())
        };
    }
}
