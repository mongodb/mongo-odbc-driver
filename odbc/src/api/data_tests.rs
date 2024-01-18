use crate::{
    api::functions::{SQLFetch, SQLMoreResults},
    handles::definitions::{
        Connection, ConnectionState, Env, EnvState, MongoHandle, Statement, StatementState,
    },
    map, set,
};
use bson::{
    doc, oid::ObjectId, spec::BinarySubtype, Binary, Bson, DateTime, JavaScriptCodeWithScope, Regex,
};
use chrono::prelude::*;
use cstr::WideChar;
use definitions::{Date, Nullability, SqlReturn, Time, Timestamp, WChar};
use lazy_static::lazy_static;
use mongo_odbc_core::{
    col_metadata::MongoColMetadata,
    json_schema::{
        simplified::{Atomic, ObjectSchema, Schema},
        BsonTypeName,
    },
    mock_query::MongoQuery,
    TypeMode,
};

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
const GUID_COL: u16 = 21;

const ARRAY_STR_VAL: (u16, &str) = (ARRAY_COL, "[1,2,3]");
const BIN_STR_VAL: (u16, &str) = (
    BIN_COL,
    "{\"$binary\":{\"base64\":\"BQYq\",\"subType\":\"00\"}}",
);
const BOOL_STR_VAL: (u16, &str) = (BOOL_COL, "true");
const DATETIME_STR_VAL: (u16, &str) = (DATETIME_COL, "{\"$date\":\"2014-11-28T12:00:09Z\"}");
const DOC_STR_VAL: (u16, &str) = (DOC_COL, "{\"x\":42,\"y\":42}");
const DOUBLE_STR_VAL: (u16, &str) = (DOUBLE_COL, "1.3");
const I32_STR_VAL: (u16, &str) = (I32_COL, "1");
const I64_STR_VAL: (u16, &str) = (I64_COL, "0");
const JS_STR_VAL: (u16, &str) = (JS_COL, "{\"$code\":\"log(\\\"hello world\\\")\"}");
const JS_W_S_STR_VAL: (u16, &str) = (
    JS_W_S_COL,
    "{\"$code\":\"log(\\\"hello\\\" + x + \\\"world\\\")\",\"$scope\":{\"x\":42}}",
);
const MAXKEY_STR_VAL: (u16, &str) = (MAXKEY_COL, "{\"$maxKey\":1}");
const MINKEY_STR_VAL: (u16, &str) = (MINKEY_COL, "{\"$minKey\":1}");
const OID_STR_VAL: (u16, &str) = (OID_COL, "{\"$oid\":\"63448dfed38427a35d534e40\"}");
const REGEX_STR_VAL: (u16, &str) = (
    REGEX_COL,
    "{\"$regularExpression\":{\"pattern\":\"hello .* world\",\"options\":\"\"}}",
);
const STRING_STR_VAL: (u16, &str) = (STRING_COL, "hello world!");
const UNIT_STR_STR_VAL: (u16, &str) = (UNIT_STR_COL, "a");
const GUID_STR_VAL: (u16, &str) = (
    GUID_COL,
    "{\"$uuid\":\"d5e3ac31-fd29-49e3-8759-08d116e98b29\"}",
);

lazy_static! {
    static ref CHRONO_TIME: chrono::DateTime<Utc> = "2014-11-28T12:00:09Z".parse().unwrap();
    static ref STANDARD_BSON_TYPE_MQ: MongoQuery = MongoQuery::new(
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
                "guid": Bson::Binary(Binary {
                    subtype: BinarySubtype::Uuid,
                    bytes: vec![0xd5, 0xe3, 0xac, 0x31, 0xfd, 0x29, 0x49, 0xe3, 0x87, 0x59, 0x08, 0xd1, 0x16, 0xe9, 0x8b, 0x29],
                }),
            }}],
            vec![
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "array".to_string(),
                    Schema::Atomic(Atomic::Array(Box::new(Schema::Atomic(Atomic::Scalar(
                        BsonTypeName::Int,
                    ))))),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "binary".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::BinData)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "bool".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Bool)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "datetime".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Date)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
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
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "f64".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Double)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3232".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3264".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Javascript)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js_w_s".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::JavascriptWithScope)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "max_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MaxKey)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "min_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MinKey)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "null".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Null)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "oid".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::ObjectId)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "regex".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Regex)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "string".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "undefined".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Undefined)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unicode".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "negative_long".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unit_str".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "guid".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::BinData)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Standard
                ),
            ],
        );

    static ref SIMPLE_BSON_TYPE_MQ: MongoQuery = MongoQuery::new(
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
                "guid": Bson::Binary(Binary {
                    subtype: BinarySubtype::Uuid,
                    bytes: vec![0xd5, 0xe3, 0xac, 0x31, 0xfd, 0x29, 0x49, 0xe3, 0x87, 0x59, 0x08, 0xd1, 0x16, 0xe9, 0x8b, 0x29],
                }),
            }}],
            vec![
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "array".to_string(),
                    Schema::Atomic(Atomic::Array(Box::new(Schema::Atomic(Atomic::Scalar(
                        BsonTypeName::Int,
                    ))))),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "binary".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::BinData)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "bool".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Bool)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "datetime".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Date)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
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
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "f64".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Double)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3232".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "i3264".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Javascript)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "js_w_s".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::JavascriptWithScope)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "max_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MaxKey)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "min_key".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::MinKey)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "null".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Null)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "oid".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::ObjectId)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "regex".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Regex)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "string".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "undefined".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Undefined)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unicode".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "negative_long".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::Long)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "unit_str".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
                MongoColMetadata::new(
                    "",
                    "test".to_string(),
                    "guid".to_string(),
                    Schema::Atomic(Atomic::Scalar(BsonTypeName::BinData)),
                    Nullability::SQL_NO_NULLS,
                    TypeMode::Simple
                ),
            ],
        );
}

fn sql_fetch_and_more_results_basic_functionality(type_mode: TypeMode) {
    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(MongoQuery::new(
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
            Nullability::SQL_NO_NULLS,
            type_mode,
        )],
    )));
    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as *mut _,));
        assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt_handle as *mut _,));
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn indicator_missing(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                CDataType::SQL_C_WCHAR as i16,
                char_buffer,
                buffer_length,
                out_len_or_ind,
            )
        );
        assert_eq!(
            "[MongoDB][API] Indicator variable was null when null data was accessed".to_string(),
            format!(
                "{}",
                (*stmt_handle)
                    .as_statement()
                    .unwrap()
                    .errors
                    .read()
                    .unwrap()[0],
            ),
        );
        let _ = Box::from_raw(char_buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_wstring_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use cstr::input_text_to_string_w;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 800])) as *mut _;
        let buffer_length: isize = 800;
        let out_len_or_ind = &mut 0;
        {
            let mut str_val_test = |col: u16, expected: &str| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_WCHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(
                    (std::mem::size_of::<WideChar>() * expected.len()) as isize,
                    *out_len_or_ind
                );
                assert_eq!(
                    expected.to_string(),
                    input_text_to_string_w(char_buffer as *const _, expected.len())
                );
            };

            str_val_test(ARRAY_STR_VAL.0, ARRAY_STR_VAL.1);
            str_val_test(BIN_STR_VAL.0, BIN_STR_VAL.1);
            str_val_test(BOOL_STR_VAL.0, BOOL_STR_VAL.1);
            str_val_test(DATETIME_STR_VAL.0, DATETIME_STR_VAL.1);
            str_val_test(DOC_STR_VAL.0, DOC_STR_VAL.1);
            str_val_test(DOUBLE_STR_VAL.0, DOUBLE_STR_VAL.1);
            str_val_test(I32_STR_VAL.0, I32_STR_VAL.1);
            str_val_test(I64_STR_VAL.0, I64_STR_VAL.1);
            str_val_test(JS_STR_VAL.0, JS_STR_VAL.1);
            str_val_test(JS_W_S_STR_VAL.0, JS_W_S_STR_VAL.1);
            str_val_test(MAXKEY_STR_VAL.0, MAXKEY_STR_VAL.1);
            str_val_test(MINKEY_STR_VAL.0, MINKEY_STR_VAL.1);
            str_val_test(OID_STR_VAL.0, OID_STR_VAL.1);
            str_val_test(REGEX_STR_VAL.0, REGEX_STR_VAL.1);
            str_val_test(STRING_STR_VAL.0, STRING_STR_VAL.1);
            str_val_test(UNIT_STR_STR_VAL.0, UNIT_STR_STR_VAL.1);
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_WCHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_WCHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(char_buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_wstring_data_by_pieces(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use cstr::input_text_to_string_w;
    use definitions::CDataType;
    use std::mem::size_of;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
        let buffer_length: isize = 3 * size_of::<WideChar>() as isize;
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
                        CDataType::SQL_C_WCHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                if code == SqlReturn::SUCCESS_WITH_INFO {
                    assert_eq!(
                        format!(
                            "[MongoDB][API] Buffer size \"{buffer_length}\" not large enough for data"
                        ),
                        format!(
                            "{}",
                            (*stmt_handle)
                                .as_statement()
                                .unwrap()
                                .errors
                                .read()
                                .unwrap()[0],
                        ),
                    );
                }
                assert_eq!(
                    std::mem::size_of::<WideChar>() as isize * expected_out_len,
                    *out_len_or_ind
                );
                assert_eq!(
                    expected.to_string(),
                    input_text_to_string_w(char_buffer as *const _, expected.chars().count())
                );
            };

            str_val_test(ARRAY_COL, 7, "[1", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 5, ",2", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 3, ",3", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 1, "]", SqlReturn::SUCCESS);
            str_val_test(ARRAY_COL, 0, "", SqlReturn::NO_DATA);

            str_val_test(UNICODE_COL, 14, "你好", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 12, "，世", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 10, "界，", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 8, "这是", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 6, "一个", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 4, "中文", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(UNICODE_COL, 2, "句子", SqlReturn::SUCCESS);
            str_val_test(UNICODE_COL, 0, "", SqlReturn::NO_DATA);
        }
        let _ = Box::from_raw(char_buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_guid_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
        let buffer_length: isize = 100;
        let out_len_or_ind = &mut 0;
        {
            let mut guid_val_test = |col: u16, expected: &[u8], code: SqlReturn| {
                assert_eq!(
                    code,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_GUID as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );

                assert_eq!(
                    expected,
                    std::slice::from_raw_parts(buffer as *const u8, expected.len()),
                );
            };

            guid_val_test(BIN_COL, &[], SqlReturn::ERROR);
            assert_eq!(
                "[MongoDB][API] BSON type binary with non-uuid subtype cannot be converted to ODBC type GUID"
                    .to_string(),
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
            guid_val_test(STRING_COL, &[], SqlReturn::ERROR);
            assert_eq!(
                "[MongoDB][API] BSON type string cannot be converted to ODBC type GUID".to_string(),
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
            guid_val_test(
                GUID_COL,
                &[
                    123, 34, 36, 117, 117, 105, 100, 34, 58, 34, 100, 53, 101, 51, 97,
                ],
                SqlReturn::SUCCESS,
            );
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_string_data_by_pieces(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use cstr::input_text_to_string_a;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let char_buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
        let buffer_length = 3;
        let out_len_or_ind = &mut 0;
        {
            let mut str_val_test =
                |col: u16, expected_out_len: isize, expected: &str, code: SqlReturn| {
                    assert_eq!(
                        code,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SQL_C_CHAR as i16,
                            char_buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    assert_eq!(expected_out_len, *out_len_or_ind);
                    assert_eq!(
                        expected.to_string(),
                        input_text_to_string_a(char_buffer as *const _, expected.chars().count())
                    );
                };

            str_val_test(ARRAY_COL, 7, "[1", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 5, ",2", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 3, ",3", SqlReturn::SUCCESS_WITH_INFO);
            str_val_test(ARRAY_COL, 1, "]", SqlReturn::SUCCESS);
            str_val_test(ARRAY_COL, 0, "", SqlReturn::NO_DATA);
        }
        let _ = Box::from_raw(char_buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_binary_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
        let buffer_length: isize = 100;
        let out_len_or_ind = &mut 0;
        {
            let mut bin_val_test = |col: u16, expected: &[u8]| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_BINARY as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    ),
                );
                assert_eq!(
                    expected.len() as isize,
                    *out_len_or_ind,
                    "expected len for column {col}"
                );
                assert_eq!(
                    expected,
                    std::slice::from_raw_parts(buffer as *const u8, expected.len()),
                    "expected contents for column {col}",
                );
            };

            bin_val_test(ARRAY_STR_VAL.0, ARRAY_STR_VAL.1.as_bytes());
            bin_val_test(BIN_COL, BIN_STR_VAL.1.as_bytes());
            bin_val_test(BOOL_COL, BOOL_STR_VAL.1.as_bytes());
            bin_val_test(DATETIME_COL, DATETIME_STR_VAL.1.as_bytes());
            bin_val_test(DOC_STR_VAL.0, DOC_STR_VAL.1.as_bytes());
            bin_val_test(DOUBLE_COL, DOUBLE_STR_VAL.1.as_bytes());
            bin_val_test(I32_COL, I32_STR_VAL.1.as_bytes());
            bin_val_test(I64_COL, I64_STR_VAL.1.as_bytes());
            bin_val_test(JS_STR_VAL.0, JS_STR_VAL.1.as_bytes());
            bin_val_test(JS_W_S_STR_VAL.0, JS_W_S_STR_VAL.1.as_bytes());
            bin_val_test(MAXKEY_STR_VAL.0, MAXKEY_STR_VAL.1.as_bytes());
            bin_val_test(MINKEY_STR_VAL.0, MINKEY_STR_VAL.1.as_bytes());
            bin_val_test(OID_STR_VAL.0, OID_STR_VAL.1.as_bytes());
            bin_val_test(REGEX_STR_VAL.0, REGEX_STR_VAL.1.as_bytes());
            bin_val_test(STRING_STR_VAL.0, STRING_STR_VAL.1.as_bytes());
            bin_val_test(UNIT_STR_STR_VAL.0, UNIT_STR_STR_VAL.1.as_bytes());
            bin_val_test(GUID_STR_VAL.0, GUID_STR_VAL.1.as_bytes());
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_BINARY as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_BINARY as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }

        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_binary_data_by_pieces(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
    unsafe {
        assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
        let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 200])) as *mut _;
        let buffer_length: isize = 25;
        let out_len_or_ind = &mut 0;
        {
            let mut bin_val_test =
                |col: u16, expected_out_len: isize, expected: &[u8], code: SqlReturn| {
                    assert_eq!(
                        code,
                        SQLGetData(
                            stmt_handle as *mut _,
                            col,
                            CDataType::SQL_C_BINARY as i16,
                            buffer,
                            buffer_length,
                            out_len_or_ind,
                        )
                    );
                    match code {
                        SqlReturn::SUCCESS | SqlReturn::SUCCESS_WITH_INFO => {
                            assert_eq!(expected_out_len, *out_len_or_ind);
                            assert_eq!(
                                expected,
                                std::slice::from_raw_parts(buffer as *const u8, expected.len())
                            );
                        }
                        _ => (),
                    }
                };
            bin_val_test(
                BIN_COL,
                44,
                &BIN_STR_VAL.1.as_bytes()[0..24],
                SqlReturn::SUCCESS_WITH_INFO,
            );
            assert_eq!(
                "[MongoDB][API] Buffer size \"25\" not large enough for data".to_string(),
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
            bin_val_test(
                BIN_COL,
                19,
                &BIN_STR_VAL.1.as_bytes()[25..],
                SqlReturn::SUCCESS,
            );
            bin_val_test(BIN_COL, 0, &[], SqlReturn::NO_DATA);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_string_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use cstr::input_text_to_string_a;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                        CDataType::SQL_C_CHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(
                    expected.len() as isize,
                    *out_len_or_ind,
                    "Expected column type {col}",
                );
                assert_eq!(
                    expected.to_string(),
                    input_text_to_string_a(char_buffer as *const _, expected.len()),
                    "Expected column type {col}",
                );
            };

            str_val_test(ARRAY_STR_VAL.0, ARRAY_STR_VAL.1);
            str_val_test(BIN_STR_VAL.0, BIN_STR_VAL.1);
            str_val_test(BOOL_STR_VAL.0, BOOL_STR_VAL.1);
            str_val_test(DATETIME_STR_VAL.0, DATETIME_STR_VAL.1);
            str_val_test(DOC_STR_VAL.0, DOC_STR_VAL.1);
            str_val_test(DOUBLE_STR_VAL.0, DOUBLE_STR_VAL.1);
            str_val_test(I32_STR_VAL.0, I32_STR_VAL.1);
            str_val_test(I64_STR_VAL.0, I64_STR_VAL.1);
            str_val_test(JS_STR_VAL.0, JS_STR_VAL.1);
            str_val_test(JS_W_S_STR_VAL.0, JS_W_S_STR_VAL.1);
            str_val_test(MAXKEY_STR_VAL.0, MAXKEY_STR_VAL.1);
            str_val_test(MINKEY_STR_VAL.0, MINKEY_STR_VAL.1);
            str_val_test(OID_STR_VAL.0, OID_STR_VAL.1);
            str_val_test(REGEX_STR_VAL.0, REGEX_STR_VAL.1);
            str_val_test(STRING_STR_VAL.0, STRING_STR_VAL.1);
            str_val_test(UNIT_STR_STR_VAL.0, UNIT_STR_STR_VAL.1);
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_CHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_CHAR as i16,
                        char_buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(char_buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_bit_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_BIT as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                    CDataType::SQL_C_BIT as i16,
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
            bool_val_test(
                JS_W_S_COL,
                false,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type Bit",
            );
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
                "[MongoDB][API] invalid character value for cast to type: Bit",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_BIT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_BIT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_i64_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_SBIGINT as i16,
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
                                    CDataType::SQL_C_SBIGINT as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid character value for cast to type: Int64",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_SBIGINT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_SBIGINT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_u64_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_UBIGINT as i16,
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
                                    CDataType::SQL_C_UBIGINT as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] BSON type array cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                BIN_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type binData cannot be converted to ODBC type UInt64",
            );
            u64_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "");
            u64_val_test(
                DATETIME_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type date cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                DOC_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type object cannot be converted to ODBC type UInt64",
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
                "[MongoDB][API] BSON type javascript cannot be converted to ODBC type UInt64",
            );
            u64_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type UInt64");
            u64_val_test(
                MAXKEY_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                MINKEY_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type minKey cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                OID_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type objectId cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                REGEX_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type regex cannot be converted to ODBC type UInt64",
            );
            u64_val_test(
                STRING_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] invalid character value for cast to type: UInt64",
            );
            u64_val_test(
                NEGATIVE_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] integral data \"-1\" was truncated due to overflow",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_UBIGINT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_UBIGINT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_i32_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_SLONG as i16,
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
                                    CDataType::SQL_C_SLONG as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid character value for cast to type: Int32",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_SLONG as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_SLONG as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_u32_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_ULONG as i16,
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
                                    CDataType::SQL_C_ULONG as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] BSON type array cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                BIN_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type binData cannot be converted to ODBC type UInt32",
            );
            u32_val_test(BOOL_COL, 1, SqlReturn::SUCCESS, "convert bool to u32");
            u32_val_test(
                DATETIME_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type date cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                DOC_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type object cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                DOUBLE_COL,
                1,
                SqlReturn::SUCCESS_WITH_INFO,
                "[MongoDB][API] floating point data \"1.3\" was truncated to fixed point",
            );
            u32_val_test(I32_COL, 1, SqlReturn::SUCCESS, "convert i32 to u32");
            u32_val_test(I64_COL, 0, SqlReturn::SUCCESS, "convert i64 to u32");
            u32_val_test(
                JS_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type javascript cannot be converted to ODBC type UInt32",
            );
            u32_val_test(JS_W_S_COL, 0, SqlReturn::ERROR, "[MongoDB][API] BSON type javascriptWithScope cannot be converted to ODBC type UInt32");
            u32_val_test(
                MAXKEY_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type maxKey cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                MINKEY_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type minKey cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                OID_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type objectId cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                REGEX_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] BSON type regex cannot be converted to ODBC type UInt32",
            );
            u32_val_test(
                STRING_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] invalid character value for cast to type: UInt32",
            );
            u32_val_test(
                NEGATIVE_COL,
                0,
                SqlReturn::ERROR,
                "[MongoDB][API] integral data \"-1\" was truncated due to overflow",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_ULONG as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_ULONG as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_f64_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_DOUBLE as i16,
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
                                    CDataType::SQL_C_DOUBLE as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid character value for cast to type: Double",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_DOUBLE as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_DOUBLE as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_f32_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_FLOAT as i16,
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
                                    CDataType::SQL_C_FLOAT as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid character value for cast to type: Double",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_FLOAT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_FLOAT as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_datetime_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_TIMESTAMP as i16,
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
                                    CDataType::SQL_C_TIMESTAMP as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid datetime format",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_TIMESTAMP as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_TIMESTAMP as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_date_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_DATE as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0],
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] datetime data \"2014-11-28 12:00:09.0 +00:00:00\" was truncated to date",
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
                "[MongoDB][API] invalid datetime format",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_DATE as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_DATE as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

fn sql_get_time_data(mq: MongoQuery) {
    use crate::api::functions::SQLGetData;
    use definitions::CDataType;

    let env = Box::into_raw(Box::new(MongoHandle::Env(Env::with_state(
        EnvState::ConnectionAllocated,
    ))));
    let conn = Box::into_raw(Box::new(MongoHandle::Connection(Connection::with_state(
        env as *mut _,
        ConnectionState::Connected,
    ))));
    let stmt = Statement::with_state(conn as *mut _, StatementState::Allocated);
    *stmt.mongo_statement.write().unwrap() = Some(Box::new(mq));

    let stmt_handle: *mut _ = &mut MongoHandle::Statement(stmt);
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
                            CDataType::SQL_C_TIME as i16,
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
                                    CDataType::SQL_C_TIME as i16,
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
                                        .errors
                                        .read()
                                        .unwrap()[0]
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
                "[MongoDB][API] invalid datetime format",
            );
        }

        {
            let mut null_val_test = |col: u16| {
                assert_eq!(
                    SqlReturn::SUCCESS,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_TIME as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
                assert_eq!(definitions::NULL_DATA, *out_len_or_ind);
                assert_eq!(
                    SqlReturn::NO_DATA,
                    SQLGetData(
                        stmt_handle as *mut _,
                        col,
                        CDataType::SQL_C_TIME as i16,
                        buffer,
                        buffer_length,
                        out_len_or_ind,
                    )
                );
            };

            null_val_test(NULL_COL);
            null_val_test(UNDEFINED_COL);
        }
        let _ = Box::from_raw(buffer as *mut WChar);
        let _ = Box::from_raw(conn as *mut WChar);
        let _ = Box::from_raw(env as *mut WChar);
    }
}

mod unit_tests {

    use super::*;
    // test unallocated_statement tests SQLFetch when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_sql_fetch() {
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
            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(
                format!("[MongoDB][API] No ResultSet"),
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
            let _ = Box::from_raw(conn as *mut WChar);
            let _ = Box::from_raw(env as *mut WChar);
        }
    }

    #[test]
    fn sql_fetch_and_more_results_basic_functionality_test() {
        sql_fetch_and_more_results_basic_functionality(TypeMode::Standard);
        sql_fetch_and_more_results_basic_functionality(TypeMode::Simple);
    }

    #[test]
    fn indicator_missing_test() {
        indicator_missing(STANDARD_BSON_TYPE_MQ.clone());
        indicator_missing(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_wstring_data_test() {
        sql_get_wstring_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_wstring_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_wstring_data_by_pieces_test() {
        sql_get_wstring_data_by_pieces(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_wstring_data_by_pieces(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_guid_data_test() {
        sql_get_guid_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_guid_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_string_data_by_pieces_test() {
        sql_get_string_data_by_pieces(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_string_data_by_pieces(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_binary_data_test() {
        sql_get_binary_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_binary_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_binary_data_by_pieces_test() {
        sql_get_binary_data_by_pieces(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_binary_data_by_pieces(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_string_data_test() {
        sql_get_string_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_string_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_bit_data_test() {
        sql_get_bit_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_bit_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_i64_data_test() {
        sql_get_i64_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_i64_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_u64_data_test() {
        sql_get_u64_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_u64_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_i32_data_test() {
        sql_get_i32_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_i32_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_u32_data_test() {
        sql_get_u32_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_u32_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_f64_data_test() {
        sql_get_f64_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_f64_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_f32_data_test() {
        sql_get_f32_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_f32_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_datetime_data_test() {
        sql_get_datetime_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_datetime_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_date_data_test() {
        sql_get_date_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_date_data(SIMPLE_BSON_TYPE_MQ.clone());
    }

    #[test]
    fn sql_get_time_data_test() {
        sql_get_time_data(STANDARD_BSON_TYPE_MQ.clone());
        sql_get_time_data(SIMPLE_BSON_TYPE_MQ.clone());
    }
}
