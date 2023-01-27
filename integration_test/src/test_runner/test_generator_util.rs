use crate::test_runner::{
    fetch_row, get_column_attribute, get_column_count, get_data, Error, Result, TestEntry,
};
use lazy_static::lazy_static;
use odbc::{safe::AutocommitOn, Allocated, NoResult, Statement};
use odbc_sys::{CDataType, Desc, SqlDataType, USmallInt};
use serde_json::{Number, Value};
use std::string::ToString;

const GENERATED_TEST_DIR: &str = "./resources/generated_test";

lazy_static! {
    static ref STRING_VAL: Value = Value::String("".to_string());
    static ref NUMBER_VAL: Value = Value::Number(Number::from(0));
}

/// Given a TestEntry and Statement, write the results of the test entry to
/// a file in the GENERATED_TEST_DIR. The only fields retained from the initial
/// TestEntry are description, db, ordered, and test_definition.
pub fn generate_baseline_test_file(
    entry: &TestEntry,
    stmt: Statement<Allocated, NoResult, AutocommitOn>,
) -> Result<()> {
    let column_count = get_column_count(&stmt)?;

    // 1. Get result set metadata
    let mut expected_catalog_name: Vec<Value> = vec![];
    let mut expected_case_sensitive: Vec<Value> = vec![];
    let mut expected_column_name: Vec<Value> = vec![];
    let mut expected_display_size: Vec<Value> = vec![];
    let mut expected_length: Vec<Value> = vec![];
    let mut expected_is_searchable: Vec<Value> = vec![];
    let mut expected_is_unsigned: Vec<Value> = vec![];
    let mut expected_sql_type: Vec<Value> = vec![];
    let mut expected_bson_type: Vec<Value> = vec![];
    let mut expected_precision: Vec<Value> = vec![];
    let mut expected_scale: Vec<Value> = vec![];
    let mut expected_nullability: Vec<Value> = vec![];

    for i in 1..(column_count + 1) {
        let catalog_name = get_column_attribute(&stmt, i, Desc::CatalogName, &STRING_VAL)?;
        expected_catalog_name.push(catalog_name);

        let case_sensitive = get_column_attribute(&stmt, i, Desc::CaseSensitive, &STRING_VAL)?;
        expected_case_sensitive.push(case_sensitive);

        let column_name = get_column_attribute(&stmt, i, Desc::Name, &STRING_VAL)?;
        expected_column_name.push(column_name);

        let display_size = get_column_attribute(&stmt, i, Desc::DisplaySize, &NUMBER_VAL)?;
        expected_display_size.push(display_size);

        let length = get_column_attribute(&stmt, i, Desc::Length, &NUMBER_VAL)?;
        expected_length.push(length);

        let is_searchable = get_column_attribute(&stmt, i, Desc::Searchable, &NUMBER_VAL)?;
        expected_is_searchable.push(is_searchable);

        let is_unsigned = get_column_attribute(&stmt, i, Desc::Unsigned, &NUMBER_VAL)?;
        expected_is_unsigned.push(is_unsigned);

        let sql_type = get_column_attribute(&stmt, i, Desc::Type, &NUMBER_VAL)?;
        expected_sql_type.push(sql_type);

        let bson_type = get_column_attribute(&stmt, i, Desc::TypeName, &STRING_VAL)?;
        expected_bson_type.push(bson_type);

        let precision = get_column_attribute(&stmt, i, Desc::Precision, &NUMBER_VAL)?;
        expected_precision.push(precision);

        let scale = get_column_attribute(&stmt, i, Desc::Scale, &NUMBER_VAL)?;
        expected_scale.push(scale);

        let nullability = get_column_attribute(&stmt, i, Desc::Nullable, &NUMBER_VAL)?;
        expected_nullability.push(nullability);
    }

    // 2. Get result set data
    let mut expected_result: Vec<Vec<Value>> = vec![];

    while fetch_row(&stmt)? {
        let mut row: Vec<Value> = vec![];
        for i in 0..(column_count) {
            let expected_data_type = get_expected_data_type(expected_sql_type.get(i).unwrap());
            let field = get_data(&stmt, i as USmallInt, expected_data_type)?;
            row.push(field);
        }
        expected_result.push(row);
    }

    // 3. Create new TestEntry with all the data
    let test_entry = TestEntry {
        description: entry.description.clone(),
        db: entry.db.clone(),
        test_definition: entry.test_definition.clone(),
        expected_result: Some(expected_result),
        skip_reason: None,
        ordered: entry.ordered,
        expected_catalog_name: Some(expected_catalog_name),
        expected_case_sensitive: Some(expected_case_sensitive),
        expected_column_name: Some(expected_column_name),
        expected_display_size: Some(expected_display_size),
        expected_length: Some(expected_length),
        expected_is_searchable: Some(expected_is_searchable),
        expected_is_unsigned: Some(expected_is_unsigned),
        expected_sql_type: Some(expected_sql_type),
        expected_bson_type: Some(expected_bson_type),
        expected_precision: Some(expected_precision),
        expected_scale: Some(expected_scale),
        expected_nullability: Some(expected_nullability),
    };

    // 4. Write the TestEntry to a file
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let desc = entry.description.clone().replace(' ', "_");
    let file_name = format!("{desc}-{now}.yml");

    let writer = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{GENERATED_TEST_DIR}/{file_name}"))
        .expect("could not open or create test file");

    serde_yaml::to_writer(writer, &test_entry).map_err(|err| Error::Yaml(err.to_string()))
}

// Get the expected CDataType for the provided sql_type.
fn get_expected_data_type(sql_type: &Value) -> CDataType {
    match sql_type {
        Value::Number(n) => {
            let sdt = SqlDataType(n.as_i64().unwrap() as i16);
            match sdt {
                SqlDataType::UNKNOWN_TYPE => CDataType::Char,
                SqlDataType::CHAR => CDataType::Char,
                SqlDataType::NUMERIC => CDataType::Numeric,
                SqlDataType::DECIMAL => CDataType::Numeric,
                SqlDataType::INTEGER => CDataType::SLong,
                SqlDataType::SMALLINT => CDataType::SShort,
                SqlDataType::FLOAT => CDataType::Float,
                SqlDataType::REAL => CDataType::Numeric,
                SqlDataType::DOUBLE => CDataType::Double,
                SqlDataType::DATETIME => CDataType::TypeTimestamp,
                SqlDataType::VARCHAR => CDataType::Char,
                SqlDataType::DATE => CDataType::TypeDate,
                SqlDataType::TIME => CDataType::TypeTime,
                SqlDataType::TIMESTAMP => CDataType::Char,
                SqlDataType::EXT_TIME_OR_INTERVAL => CDataType::Char,
                SqlDataType::EXT_TIMESTAMP => CDataType::Default,
                SqlDataType::EXT_LONG_VARCHAR => CDataType::Char,
                SqlDataType::EXT_BINARY => CDataType::Binary,
                SqlDataType::EXT_VAR_BINARY => CDataType::Binary,
                SqlDataType::EXT_LONG_VAR_BINARY => CDataType::Binary,
                SqlDataType::EXT_BIG_INT => CDataType::SBigInt,
                SqlDataType::EXT_TINY_INT => CDataType::STinyInt,
                SqlDataType::EXT_BIT => CDataType::Bit,
                SqlDataType::EXT_W_CHAR => CDataType::WChar,
                SqlDataType::EXT_W_VARCHAR => CDataType::WChar,
                SqlDataType::EXT_W_LONG_VARCHAR => CDataType::WChar,
                SqlDataType::EXT_GUID => CDataType::Guid,
                v => unreachable!("invalid sql_type encountered: {:?}", v),
            }
        }
        v => unreachable!("sql_type should always be a number: {:?}", v),
    }
}
