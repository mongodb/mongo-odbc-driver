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
const UNICODE_COL: u16 = 18;
const NEGATIVE_COL: u16 = 19;
const UNIT_STR_COL: u16 = 20;

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
                "f64": 1.3,
                "i3232": Bson::Int32(1i32),
                "i3264": Bson::Int64(0i64),
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
                "unicode": "你好，世界，这是一个中文句子",
                "negative_long": -1i64,
                "unit_str": "a",
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
                    "f64".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Double)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3232".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3264".to_string(),
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
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unicode".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "negative_long".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    ColumnNullability::NoNulls,
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unit_str".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
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
    fn indicator_missing() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 100;
            let out_len_or_ind = std::ptr::null_mut();
            assert_eq!(
                SqlReturn::SUCCESS_WITH_INFO,
                SQLGetData(
                    stmt_handle as *mut _,
                    NULL_COL,
                    CDataType::WChar,
                    char_buffer,
                    buffer_length,
                    out_len_or_ind,
                )
            );
            assert_eq!(
                "[MongoDB][API] Indicator variable was null when null data was accessed"
                    .to_string(),
                format!(
                    "{}",
                    (*stmt_handle)
                        .as_statement()
                        .unwrap()
                        .read()
                        .unwrap()
                        .errors[0],
                ),
            );
            let _ = Box::from_raw(char_buffer);
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
                str_val_test(DOUBLE_COL, "{\"$numberDouble\":\"1.3\"}");
                str_val_test(I32_COL, "{\"$numberInt\":\"1\"}");
                str_val_test(I64_COL, "{\"$numberLong\":\"0\"}");
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
                str_val_test(UNIT_STR_COL, "a");
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::WChar,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_wstring_data_by_pieces() {
        use crate::api::{data::input_wtext_to_string, functions::SQLGetData};
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut str_val_test = |col: u16,
                                        expected_out_len: isize,
                                        expected: &str,
                                        code: SqlReturn| {
                    assert_eq!(
                        code,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::WChar,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    if code == SqlReturn::SUCCESS_WITH_INFO {
                        assert_eq!(
                            format!(
                                "[MongoDB][API] Buffer size \"{}\" not large enough for data",
                                buffer_length
                            ),
                            format!(
                                "{}",
                                (*stmt_handle)
                                    .as_statement()
                                    .unwrap()
                                    .read()
                                    .unwrap()
                                    .errors[0],
                            ),
                        );
                    }
                    assert_eq!(expected_out_len, *out_len_or_ind);
                    assert_eq!(
                        expected.to_string(),
                        input_wtext_to_string(char_buffer as *const _, expected.chars().count())
                    );
                    assert_eq!(
                        expected.to_string(),
                        input_wtext_to_string(char_buffer as *const _, expected.chars().count())
                    );
                };

                str_val_test(ARRAY_COL, 58, "[{\"$numbe", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 49, "rInt\":\"1\"", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 40, "},{\"$numb", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 31, "erInt\":\"2", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 22, "\"},{\"$num", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 13, "berInt\":\"", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 4, "3\"}]", SqlReturn::SUCCESS);
                str_val_test(ARRAY_COL, 0, "", SqlReturn::NO_DATA);

                str_val_test(
                    UNICODE_COL,
                    14,
                    "你好，世界，这是一",
                    SqlReturn::SUCCESS_WITH_INFO,
                );
                str_val_test(UNICODE_COL, 5, "个中文句子", SqlReturn::SUCCESS);
                str_val_test(UNICODE_COL, 0, "", SqlReturn::NO_DATA);
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_string_data_by_pieces() {
        use crate::api::{data::input_text_to_string, functions::SQLGetData};
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 10;
            let out_len_or_ind = &mut 0;
            {
                let mut str_val_test =
                    |col: u16, expected_out_len: isize, expected: &str, code: SqlReturn| {
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Char,
                                char_buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        assert_eq!(expected_out_len, *out_len_or_ind);
                        assert_eq!(
                            expected.to_string(),
                            input_text_to_string(char_buffer as *const _, expected.chars().count())
                        );
                    };

                str_val_test(ARRAY_COL, 58, "[{\"$numbe", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 49, "rInt\":\"1\"", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 40, "},{\"$numb", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 31, "erInt\":\"2", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 22, "\"},{\"$num", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 13, "berInt\":\"", SqlReturn::SUCCESS_WITH_INFO);
                str_val_test(ARRAY_COL, 4, "3\"}]", SqlReturn::SUCCESS);
                str_val_test(ARRAY_COL, 0, "", SqlReturn::NO_DATA);
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_binary_data_by_pieces() {
        use crate::api::functions::SQLGetData;
        use odbc_sys::CDataType;
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new((*MQ).clone()));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
            let buffer_length: isize = 2;
            let out_len_or_ind = &mut 0;
            {
                let mut bin_val_test =
                    |col: u16, expected_out_len: isize, expected: &[u8], code: SqlReturn| {
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Binary,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        assert_eq!(expected_out_len, *out_len_or_ind);
                        assert_eq!(
                            expected,
                            std::slice::from_raw_parts(buffer as *const u8, expected.len()),
                        );
                    };

                bin_val_test(BIN_COL, 3, &[5u8, 6u8], SqlReturn::SUCCESS_WITH_INFO);
                assert_eq!(
                    "[MongoDB][API] Buffer size \"2\" not large enough for data".to_string(),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .read()
                            .unwrap()
                            .errors[0]
                    ),
                );
                bin_val_test(BIN_COL, 1, &[42u8], SqlReturn::SUCCESS);
                bin_val_test(BIN_COL, 0, &[], SqlReturn::NO_DATA);

                assert_eq!(
                    SqlReturn::ERROR,
                    SQLGetData(
                        stmt_handle as *mut _,
                        UNICODE_COL,
                        CDataType::Binary,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(
                    "[MongoDB][API] BSON type string cannot be converted to ODBC type Binary"
                        .to_string(),
                    format!(
                        "{}",
                        (*stmt_handle)
                            .as_statement()
                            .unwrap()
                            .read()
                            .unwrap()
                            .errors[1]
                    ),
                );
            }
            let _ = Box::from_raw(buffer);
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
                str_val_test(DOUBLE_COL, "{\"$numberDouble\":\"1.3\"}");
                str_val_test(I32_COL, "{\"$numberInt\":\"1\"}");
                str_val_test(I64_COL, "{\"$numberLong\":\"0\"}");
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
                str_val_test(UNIT_STR_COL, "a");
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Char,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(char_buffer);
        }
    }

    #[test]
    fn sql_get_bit_data() {
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
                let mut bool_val_test =
                    |col: u16, expected: bool, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Bit,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0]
                                    )
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(1, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const bool));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0]
                                    )
                                );
                            }
                            SqlReturn::SUCCESS => {
                                assert_eq!(1, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const bool));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::Bit,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            _ => {
                                panic!();
                            }
                        }
                    };

                bool_val_test(
                    ARRAY_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    BIN_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Bit",
                );
                bool_val_test(BOOL_COL, true, SqlReturn::SUCCESS, "");
                bool_val_test(
                    DATETIME_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    DOC_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    DOUBLE_COL,
                    true,
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
                );
                bool_val_test(I32_COL, true, SqlReturn::SUCCESS, "");
                bool_val_test(I64_COL, false, SqlReturn::SUCCESS, "");
                bool_val_test(
                    JS_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Bit",
                );
                bool_val_test(JS_W_S_COL, false, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Bit");
                bool_val_test(
                    MAXKEY_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    MINKEY_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    OID_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    REGEX_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Bit",
                );
                bool_val_test(
                    STRING_COL,
                    false,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Bit",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Bit,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_i64_data() {
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
                let mut i64_val_test =
                    |col: u16, expected: i64, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::SBigInt,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(8, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const i64));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::SBigInt,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(8, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const i64));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                i64_val_test(
                    ARRAY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    BIN_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Int64",
                );
                i64_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "");
                i64_val_test(
                    DATETIME_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    DOC_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    DOUBLE_COL,
                    1,
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
                );
                i64_val_test(I32_COL, 1, SqlReturn::SUCCESS, "");
                i64_val_test(I64_COL, 0, SqlReturn::SUCCESS, "");
                i64_val_test(
                    JS_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Int64",
                );
                i64_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Int64");
                i64_val_test(
                    MAXKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    MINKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    OID_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    REGEX_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Int64",
                );
                i64_val_test(
                    STRING_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Int64",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SBigInt,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_u64_data() {
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
                let mut u64_val_test =
                    |col: u16, expected: u64, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::UBigInt,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(8, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const u64));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::UBigInt,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(8, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const u64));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                u64_val_test(
                    ARRAY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    BIN_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Int64",
                );
                u64_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "");
                u64_val_test(
                    DATETIME_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    DOC_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    DOUBLE_COL,
                    1,
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
                );
                u64_val_test(I32_COL, 1, SqlReturn::SUCCESS, "");
                u64_val_test(I64_COL, 0, SqlReturn::SUCCESS, "");
                u64_val_test(
                    JS_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Int64",
                );
                u64_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Int64");
                u64_val_test(
                    MAXKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    MINKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    OID_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    REGEX_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Int64",
                );
                u64_val_test(
                    STRING_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Int64",
                );
                u64_val_test(
                    NEGATIVE_COL,
                    18446744073709551615u64,
                    SqlReturn::SUCCESS,
                    "",
                );
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::UBigInt,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::UBigInt,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_i32_data() {
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
                let mut i32_val_test =
                    |col: u16, expected: i32, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::SLong,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(4, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const i32));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::SLong,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(4, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const i32));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                i32_val_test(
                    ARRAY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    BIN_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Int32",
                );
                i32_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "");
                i32_val_test(
                    DATETIME_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    DOC_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    DOUBLE_COL,
                    1,
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
                );
                i32_val_test(I32_COL, 1, SqlReturn::SUCCESS, "");
                i32_val_test(I64_COL, 0, SqlReturn::SUCCESS, "");
                i32_val_test(
                    JS_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Int32",
                );
                i32_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Int32");
                i32_val_test(
                    MAXKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    MINKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    OID_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    REGEX_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Int32",
                );
                i32_val_test(
                    STRING_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Int32",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SLong,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_u32_data() {
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
                let mut u32_val_test =
                    |col: u16, expected: u32, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::ULong,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(4, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const u32));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::ULong,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(4, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const u32));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                u32_val_test(
                    ARRAY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    BIN_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Int32",
                );
                u32_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "");
                u32_val_test(
                    DATETIME_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    DOC_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    DOUBLE_COL,
                    1,
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
                );
                u32_val_test(I32_COL, 1, SqlReturn::SUCCESS, "");
                u32_val_test(I64_COL, 0, SqlReturn::SUCCESS, "");
                u32_val_test(
                    JS_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Int32",
                );
                u32_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Int32");
                u32_val_test(
                    MAXKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    MINKEY_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    OID_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    REGEX_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Int32",
                );
                u32_val_test(
                    STRING_COL,
                    0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Int32",
                );
                u32_val_test(NEGATIVE_COL, 4294967295, SqlReturn::SUCCESS, "");
            }

            {
                let mut null_val_test = |col: u16| {
                    assert_eq!(
                        SqlReturn::SUCCESS,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::ULong,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(odbc_sys::NULL_DATA, *out_len_or_ind);
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::ULong,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_f64_data() {
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
                let mut f64_val_test =
                    |col: u16, expected: f64, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Double,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(8, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const f64));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::Double,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                f64_val_test(
                    ARRAY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    BIN_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Double",
                );
                f64_val_test(BOOL_COL, 1.0, SqlReturn::SUCCESS, "");
                f64_val_test(
                    DATETIME_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    DOC_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Double",
                );
                f64_val_test(DOUBLE_COL, 1.3, SqlReturn::SUCCESS, "");
                f64_val_test(I32_COL, 1.0, SqlReturn::SUCCESS, "");
                f64_val_test(I64_COL, 0.0, SqlReturn::SUCCESS, "");
                f64_val_test(
                    JS_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Double",
                );
                f64_val_test(JS_W_S_COL, 0.0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Double");
                f64_val_test(
                    MAXKEY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    MINKEY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    OID_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    REGEX_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Double",
                );
                f64_val_test(
                    STRING_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Double",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Double,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }

    #[test]
    fn sql_get_f32_data() {
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
                let mut f32_val_test =
                    |col: u16, expected: f32, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Float,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(4, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const f32));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::Float,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            _ => panic!(),
                        }
                    };

                f32_val_test(
                    ARRAY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    BIN_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type Double",
                );
                f32_val_test(BOOL_COL, 1.0, SqlReturn::SUCCESS, "");
                f32_val_test(
                    DATETIME_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type date cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    DOC_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type Double",
                );
                f32_val_test(DOUBLE_COL, 1.3, SqlReturn::SUCCESS, "");
                f32_val_test(I32_COL, 1.0, SqlReturn::SUCCESS, "");
                f32_val_test(I64_COL, 0.0, SqlReturn::SUCCESS, "");
                f32_val_test(
                    JS_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type Double",
                );
                f32_val_test(JS_W_S_COL, 0.0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Double");
                f32_val_test(
                    MAXKEY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    MINKEY_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    OID_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    REGEX_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type Double",
                );
                f32_val_test(
                    STRING_COL,
                    0.0,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid character value: \"hello world!\" for cast to type: Double",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Float,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
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
                let mut datetime_val_test =
                    |col: u16, expected: Timestamp, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::TimeStamp,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(16, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const Timestamp));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::TimeStamp,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0]
                                    )
                                );
                            }
                            _ => {
                                panic!()
                            }
                        }
                    };

                let empty = Timestamp {
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    fraction: 0,
                };
                datetime_val_test(
                    ARRAY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    BIN_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    BOOL_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type bool cannot be converted to ODBC type DateTime",
                );
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
                    SqlReturn::SUCCESS,
                    "",
                );
                datetime_val_test(
                    DOC_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    DOUBLE_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type double cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    I32_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type int cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    I64_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type long cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    JS_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(JS_W_S_COL, empty, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type DateTime");
                datetime_val_test(
                    MAXKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    MINKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    OID_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    REGEX_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type DateTime",
                );
                datetime_val_test(
                    STRING_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid datetime format: \"hello world!\"",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::TimeStamp,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
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
                let mut date_val_test =
                    |col: u16, expected: Date, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Date,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS_WITH_INFO => {
                                assert_eq!(6, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const Date));
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0],
                                    ),
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0]
                                    )
                                );
                            }
                            _ => {
                                panic!()
                            }
                        }
                    };

                let empty = Date {
                    year: 0,
                    month: 0,
                    day: 0,
                };
                date_val_test(
                    ARRAY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    BIN_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    BOOL_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type bool cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    DATETIME_COL,
                    Date {
                        year: 2014,
                        month: 11,
                        day: 28,
                    },
                    SqlReturn::SUCCESS_WITH_INFO,
                    "[MongoDB][API] floating point data \"2014-11-28 12:00:09.0 +00:00:00\" was truncated to fixed point",
                );
                date_val_test(
                    DOC_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    DOUBLE_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type double cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    I32_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type int cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    I64_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type long cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    JS_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type DateTime",
                );
                date_val_test(JS_W_S_COL, empty, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type DateTime");
                date_val_test(
                    MAXKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    MINKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    OID_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    REGEX_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type DateTime",
                );
                date_val_test(
                    STRING_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid datetime format: \"hello world!\"",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Date,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
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
                let mut time_val_test =
                    |col: u16, expected: Time, code: SqlReturn, expected_error: &str| {
                        stmt_handle.as_mut().unwrap().clear_diagnostics();
                        assert_eq!(
                            code,
                            SQLGetData(
                                stmt_handle as *mut _,
                                col,
                                CDataType::Time,
                                buffer,
                                buffer_length,
                                out_len_or_ind,
                            )
                        );
                        match code {
                            SqlReturn::SUCCESS => {
                                assert_eq!(6, *out_len_or_ind);
                                assert_eq!(expected, *(buffer as *const Time));
                                assert_eq!(
                                    SqlReturn::NO_DATA,
                                    SQLGetData(
                                        stmt_handle as *mut _,
                                        col,
                                        CDataType::Time,
                                        buffer,
                                        buffer_length,
                                        out_len_or_ind,
                                    )
                                );
                            }
                            SqlReturn::ERROR => {
                                assert_eq!(
                                    expected_error.to_string(),
                                    format!(
                                        "{}",
                                        (*stmt_handle)
                                            .as_statement()
                                            .unwrap()
                                            .read()
                                            .unwrap()
                                            .errors[0]
                                    )
                                );
                            }
                            _ => {
                                panic!()
                            }
                        }
                    };

                let empty = Time {
                    hour: 0,
                    minute: 0,
                    second: 0,
                };
                time_val_test(
                    ARRAY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type array cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    BIN_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type binData cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    BOOL_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type bool cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    DATETIME_COL,
                    Time {
                        hour: 12,
                        minute: 0,
                        second: 9,
                    },
                    SqlReturn::SUCCESS,
                    "",
                );
                time_val_test(
                    DOC_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type object cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    DOUBLE_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type double cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    I32_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type int cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    I64_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type long cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    JS_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type javascript cannot be converted to ODBC type DateTime",
                );
                time_val_test(JS_W_S_COL, empty, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type DateTime");
                time_val_test(
                    MAXKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    MINKEY_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type minKey cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    OID_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type objectId cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    REGEX_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] BSON type regex cannot be converted to ODBC type DateTime",
                );
                time_val_test(
                    STRING_COL,
                    empty,
                    SqlReturn::ERROR,
                    "[MongoDB][API] invalid datetime format: \"hello world!\"",
                );
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
                    assert_eq!(
                        SqlReturn::NO_DATA,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::Time,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                };

                null_val_test(NULL_COL);
                null_val_test(UNDEFINED_COL);
            }
            let _ = Box::from_raw(buffer);
        }
    }
}
