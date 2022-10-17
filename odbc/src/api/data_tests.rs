use crate::{
    api::functions::{SQLFetch, SQLMoreResults},
    handles::definitions::{MongoHandle, Statement, StatementState},
    map, set,
};
use bson::{
    doc, oid::ObjectId, spec::BinarySubtype, Binary, Bson, DateTime, JavaScriptCodeWithScope, Regex,
};
use chrono::prelude::*;
use lazy_static::lazy_static;
use mongo_odbc_core::{
    col_metadata::{ColumnNullability, MongoColMetadata},
    json_schema::{
        simplified::{Atomic, ObjectSchema, Schema},
        BsonTypeName,
    },
    mock_query::MongoQuery,
};
use odbc_sys::{Date, SqlReturn, Time, Timestamp};
use std::sync::RwLock;

const ARRAY_COL: u16 = 1;
const BIN_COL: u16 = 2;
const BOOL_COL: u16 = 3;
const DATETIME_COL: u16 = 4;
const DOC_COL: u16 = 5;
const DOUBLE_COL: u16 = 6;
const I32_COL: u16 = 7;
const I64_COL: u16 = 8;
const JS_COL: u16 = 9;
const JS_W_S_COL: u16 = 10;
const MAXKEY_COL: u16 = 11;
const MINKEY_COL: u16 = 12;
const NULL_COL: u16 = 13;
const OID_COL: u16 = 14;
const REGEX_COL: u16 = 15;
const STRING_COL: u16 = 16;
const UNDEFINED_COL: u16 = 17;

lazy_static! {
    static ref CHRONO_TIME: chrono::DateTime<Utc> = "2014-11-28T12:00:09Z".parse().unwrap();
    static ref MQ: MongoQuery = MongoQuery::new(
            vec![doc! {"test": {
                "array": [1i32, 2i32, 3i32],
                "binary": Bson::Binary(Binary {
                    subtype: BinarySubtype::Generic,
                    bytes: vec![5u8, 6u8, 42u8],
                }),
                "bool": true,
                "datetime": Bson::DateTime(DateTime::from_chrono(*CHRONO_TIME)),
                // no good way to easily test dbpointer.
                // TODO: SQL-1068: Add Decimal128 value.
                "doc": {"x": 42i32, "y": 42i32},
                "double": 42.0,
                "int32": Bson::Int32(42i32),
                "int64": Bson::Int64(42i64),
                "js": Bson::JavaScriptCode("log(\"hello world\")".to_string()),
                "js_w_s": Bson::JavaScriptCodeWithScope(
                        JavaScriptCodeWithScope {
                            code: "log(\"hello\" + x + \"world\")".to_string(),
                            scope: doc!{"x": 42},
                        },
                    ),
                "max_key": Bson::MaxKey,
                "min_key": Bson::MinKey,
                "null": null,
                "oid": Bson::ObjectId(ObjectId::parse_str("63448dfed38427a35d534e40").unwrap()),
                "regex": Bson::RegularExpression(Regex{
                    pattern: "hello .* world".to_string(),
                    options: "".to_string()
                }),
                "string": "hello world!",
                "undefined": Bson::Undefined,
            }}],
            vec![
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "array".to_string(),
                    Schema::Atomic(Atomic::Array(Box::new(Schema::Atomic(Atomic::Scalar(
                        BsonTypeName::Int,
                    ))))),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "binary".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::BinData)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "bool".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Bool)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "datetime".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Date)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "doc".to_string(),
                    Schema::Atomic(Atomic::Object(ObjectSchema {
                        properties: map! {
                            "x".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                            "y".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                        },
                        required: set! {"x".to_string(), "y".to_string()},
                        additional_properties: false,
                    })),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "double".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Double)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "int32".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "int64".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Javascript)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js_w_s".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::JavascriptWithScope)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "max_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MaxKey)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "min_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MinKey)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "null".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Null)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "oid".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::ObjectId)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "regex".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Regex)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "string".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "undefined".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Undefined)),
                    ColumnNullability::NoNulls,
                ),
            ],
        );
}

mod unit {
    use super::*;
    // test unallocated_statement tests SQLFetch when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_sql_fetch() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

        unsafe {
            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(
                format!("[MongoDB][API] No ResultSet"),
                format!(
                    "{}",
                    (*stmt_handle)
                        .as_statement()
                        .unwrap()
                        .read()
                        .unwrap()
                        .errors[0]
                ),
            )
        }
    }

    #[test]
    fn sql_fetch_and_more_results_basic_functionality() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new(MongoQuery::new(
            vec![
                doc! {"a": {"b": 42}},
                doc! {"a": {"b": 43}},
                doc! {"a": {"b": 44}},
            ],
            vec![MongoColMetadata::new(
                "",
                "a".to_string(),
                "b".to_string(),
                Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                ColumnNullability::NoNulls,
            )],
        )));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt_handle as *mut _,));
        }
    }

    #[test]
    fn sql_get_wstring_data() {
        use crate::api::{data::input_wtext_to_string, functions::SQLGetData};
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 100;
            let out_len_or_ind = &mut 0;
            {
                let mut str_val_test = |col: u16, expected: &str| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::WChar,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(expected.len() as isize, *out_len_or_ind);
                    assert_eq!(
                        expected.to_string(),
                        input_wtext_to_string(char_buffer as *const _, expected.len())
                    );
                };

                str_val_test(
                    ARRAY_COL,
                    "[{\"$numberInt\":\"1\"},{\"$numberInt\":\"2\"},{\"$numberInt\":\"3\"}]",
                );
                str_val_test(
                    BIN_COL,
                    "{\"$binary\":{\"base64\":\"BQYq\",\"subType\":\"00\"}}",
                );
                str_val_test(BOOL_COL, "true");
                str_val_test(
                    DATETIME_COL,
                    "{\"$date\":{\"$numberLong\":\"1417176009000\"}}",
                );
                str_val_test(
                    DOC_COL,
                    "{\"x\":{\"$numberInt\":\"42\"},\"y\":{\"$numberInt\":\"42\"}}",
                );
                str_val_test(DOUBLE_COL, "{\"$numberDouble\":\"42.0\"}");
                str_val_test(I32_COL, "{\"$numberInt\":\"42\"}");
                str_val_test(I64_COL, "{\"$numberLong\":\"42\"}");
                str_val_test(JS_COL, "{\"$code\":\"log(\\\"hello world\\\")\"}");
                str_val_test(JS_W_S_COL, "{\"$code\":\"log(\\\"hello\\\" + x + \\\"world\\\")\",\"$scope\":{\"x\":{\"$numberInt\":\"42\"}}}");
                str_val_test(MAXKEY_COL, "{\"$maxKey\":1}");
                str_val_test(MINKEY_COL, "{\"$minKey\":1}");
                str_val_test(OID_COL, "{\"$oid\":\"63448dfed38427a35d534e40\"}");
                str_val_test(
                    REGEX_COL,
                    "{\"$regularExpression\":{\"pattern\":\"hello .* world\",\"options\":\"\"}}",
                );
                str_val_test(STRING_COL, "hello world!");
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::WChar,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL); // null
                null_val_test(UNDEFINED_COL); // undefined
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_string_data() {
        use crate::api::{data::input_text_to_string, functions::SQLGetData};
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 100;
            let out_len_or_ind = &mut 0;
            {
                let mut str_val_test = |col: u16, expected: &str| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Char,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(expected.len() as isize, *out_len_or_ind);
                    assert_eq!(
                        expected.to_string(),
                        input_text_to_string(char_buffer as *const _, expected.len())
                    );
                };

                str_val_test(
                    ARRAY_COL,
                    "[{\"$numberInt\":\"1\"},{\"$numberInt\":\"2\"},{\"$numberInt\":\"3\"}]",
                );
                str_val_test(
                    BIN_COL,
                    "{\"$binary\":{\"base64\":\"BQYq\",\"subType\":\"00\"}}",
                );
                str_val_test(BOOL_COL, "true");
                str_val_test(
                    DATETIME_COL,
                    "{\"$date\":{\"$numberLong\":\"1417176009000\"}}",
                );
                str_val_test(
                    DOC_COL,
                    "{\"x\":{\"$numberInt\":\"42\"},\"y\":{\"$numberInt\":\"42\"}}",
                );
                str_val_test(DOUBLE_COL, "{\"$numberDouble\":\"42.0\"}");
                str_val_test(I32_COL, "{\"$numberInt\":\"42\"}");
                str_val_test(I64_COL, "{\"$numberLong\":\"42\"}");
                str_val_test(JS_COL, "{\"$code\":\"log(\\\"hello world\\\")\"}");
                str_val_test(JS_W_S_COL, "{\"$code\":\"log(\\\"hello\\\" + x + \\\"world\\\")\",\"$scope\":{\"x\":{\"$numberInt\":\"42\"}}}");
                str_val_test(MAXKEY_COL, "{\"$maxKey\":1}");
                str_val_test(MINKEY_COL, "{\"$minKey\":1}");
                str_val_test(OID_COL, "{\"$oid\":\"63448dfed38427a35d534e40\"}");
                str_val_test(
                    REGEX_COL,
                    "{\"$regularExpression\":{\"pattern\":\"hello .* world\",\"options\":\"\"}}",
                );
                str_val_test(STRING_COL, "hello world!");
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Char,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_bool_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 10])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut bool_val_test = |col: u16, expected: bool| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Bit,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(1, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const bool));
                };

                bool_val_test(ARRAY_COL, false);
                bool_val_test(BIN_COL, false);
                bool_val_test(BOOL_COL, true);
                bool_val_test(DATETIME_COL, true);
                bool_val_test(DOC_COL, false);
                bool_val_test(DOUBLE_COL, true);
                bool_val_test(I32_COL, true);
                bool_val_test(I64_COL, true);
                bool_val_test(JS_COL, false);
                bool_val_test(JS_W_S_COL, false);
                bool_val_test(MAXKEY_COL, false);
                bool_val_test(MINKEY_COL, false);
                bool_val_test(OID_COL, false);
                bool_val_test(REGEX_COL, false);
                bool_val_test(STRING_COL, true);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Bit,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_long_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 10])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut long_val_test = |col: u16, expected: i64| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SBigInt,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(8, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const i64));
                };

                long_val_test(ARRAY_COL, 0);
                long_val_test(BIN_COL, 0);
                long_val_test(BOOL_COL, 1);
                long_val_test(DATETIME_COL, 1417176009000);
                long_val_test(DOC_COL, 0);
                long_val_test(DOUBLE_COL, 42);
                long_val_test(I32_COL, 42);
                long_val_test(I64_COL, 42);
                long_val_test(JS_COL, 0);
                long_val_test(JS_W_S_COL, 0);
                long_val_test(MAXKEY_COL, 0);
                long_val_test(MINKEY_COL, 0);
                long_val_test(OID_COL, 0);
                long_val_test(REGEX_COL, 0);
                long_val_test(STRING_COL, 0);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SBigInt,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_int_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 10])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut int_val_test = |col: u16, expected: i32| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SLong,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(4, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const i32));
                };

                int_val_test(ARRAY_COL, 0);
                int_val_test(BIN_COL, 0);
                int_val_test(BOOL_COL, 1);
                int_val_test(DATETIME_COL, -163198680);
                int_val_test(DOC_COL, 0);
                int_val_test(DOUBLE_COL, 42);
                int_val_test(I32_COL, 42);
                int_val_test(I64_COL, 42);
                int_val_test(JS_COL, 0);
                int_val_test(JS_W_S_COL, 0);
                int_val_test(MAXKEY_COL, 0);
                int_val_test(MINKEY_COL, 0);
                int_val_test(OID_COL, 0);
                int_val_test(REGEX_COL, 0);
                int_val_test(STRING_COL, 0);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SLong,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_double_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 10])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut double_val_test = |col: u16, expected: f64| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Double,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(8, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const f64));
                };

                double_val_test(ARRAY_COL, 0.0);
                double_val_test(BIN_COL, 0.0);
                double_val_test(BOOL_COL, 1.0);
                double_val_test(DATETIME_COL, 1417176009000.0);
                double_val_test(DOC_COL, 0.0);
                double_val_test(DOUBLE_COL, 42.0);
                double_val_test(I32_COL, 42.0);
                double_val_test(I64_COL, 42.0);
                double_val_test(JS_COL, 0.0);
                double_val_test(JS_W_S_COL, 0.0);
                double_val_test(MAXKEY_COL, 0.0);
                double_val_test(MINKEY_COL, 0.0);
                double_val_test(OID_COL, 0.0);
                double_val_test(REGEX_COL, 0.0);
                double_val_test(STRING_COL, 0.0);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Double,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_float_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 10])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut float_val_test = |col: u16, expected: f32| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Float,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(4, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const f32));
                };

                float_val_test(ARRAY_COL, 0.0);
                float_val_test(BIN_COL, 0.0);
                float_val_test(BOOL_COL, 1.0);
                float_val_test(DATETIME_COL, 1417176009000.0);
                float_val_test(DOC_COL, 0.0);
                float_val_test(DOUBLE_COL, 42.0);
                float_val_test(I32_COL, 42.0);
                float_val_test(I64_COL, 42.0);
                float_val_test(JS_COL, 0.0);
                float_val_test(JS_W_S_COL, 0.0);
                float_val_test(MAXKEY_COL, 0.0);
                float_val_test(MINKEY_COL, 0.0);
                float_val_test(OID_COL, 0.0);
                float_val_test(REGEX_COL, 0.0);
                float_val_test(STRING_COL, 0.0);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Float,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_datetime_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let buffer_length: isize = 40;
            let out_len_or_ind = &mut 0;
            {
                let mut datetime_val_test = |col: u16, expected: Timestamp| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::TimeStamp,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(16, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const Timestamp));
                };

                let empty = Timestamp {
                    year: 1970,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    fraction: 0,
                };
                datetime_val_test(ARRAY_COL, empty);
                datetime_val_test(BIN_COL, empty);
                datetime_val_test(BOOL_COL, empty);
                datetime_val_test(
                    DATETIME_COL,
                    Timestamp {
                        year: 2014,
                        month: 11,
                        day: 28,
                        hour: 12,
                        minute: 0,
                        second: 9,
                        fraction: 0,
                    },
                );
                datetime_val_test(DOC_COL, empty);
                let forty_two = Timestamp {
                    year: 1970,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    fraction: 42,
                };
                datetime_val_test(DOUBLE_COL, forty_two);
                datetime_val_test(I32_COL, forty_two);
                datetime_val_test(I64_COL, forty_two);
                datetime_val_test(JS_COL, empty);
                datetime_val_test(JS_W_S_COL, empty);
                datetime_val_test(MAXKEY_COL, empty);
                datetime_val_test(MINKEY_COL, empty);
                datetime_val_test(OID_COL, empty);
                datetime_val_test(REGEX_COL, empty);
                datetime_val_test(STRING_COL, empty);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::TimeStamp,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_date_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let buffer_length: isize = 40;
            let out_len_or_ind = &mut 0;
            {
                let mut date_val_test = |col: u16, expected: Date| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Date,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(6, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const Date));
                };

                let empty = Date {
                    year: 1970,
                    month: 1,
                    day: 1,
                };
                date_val_test(ARRAY_COL, empty);
                date_val_test(BIN_COL, empty);
                date_val_test(BOOL_COL, empty);
                date_val_test(
                    DATETIME_COL,
                    Date {
                        year: 2014,
                        month: 11,
                        day: 28,
                    },
                );
                date_val_test(DOC_COL, empty);
                date_val_test(DOUBLE_COL, empty);
                date_val_test(I32_COL, empty);
                date_val_test(I64_COL, empty);
                date_val_test(JS_COL, empty);
                date_val_test(JS_W_S_COL, empty);
                date_val_test(MAXKEY_COL, empty);
                date_val_test(MINKEY_COL, empty);
                date_val_test(OID_COL, empty);
                date_val_test(REGEX_COL, empty);
                date_val_test(STRING_COL, empty);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Date,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_time_data() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 40])) as *mut _;
            let buffer_length: isize = 40;
            let out_len_or_ind = &mut 0;
            {
                let mut time_val_test = |col: u16, expected: Time| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Time,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(6, *out_len_or_ind);
                    assert_eq!(expected, *(buffer as *const Time));
                };

                let empty = Time {
                    hour: 0,
                    minute: 0,
                    second: 0,
                };
                time_val_test(ARRAY_COL, empty);
                time_val_test(BIN_COL, empty);
                time_val_test(BOOL_COL, empty);
                time_val_test(
                    DATETIME_COL,
                    Time {
                        hour: 12,
                        minute: 0,
                        second: 9,
                    },
                );
                time_val_test(DOC_COL, empty);
                time_val_test(DOUBLE_COL, empty);
                time_val_test(I32_COL, empty);
                time_val_test(I64_COL, empty);
                time_val_test(JS_COL, empty);
                time_val_test(JS_W_S_COL, empty);
                time_val_test(MAXKEY_COL, empty);
                time_val_test(MINKEY_COL, empty);
                time_val_test(OID_COL, empty);
                time_val_test(REGEX_COL, empty);
                time_val_test(STRING_COL, empty);
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Time,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }
}
