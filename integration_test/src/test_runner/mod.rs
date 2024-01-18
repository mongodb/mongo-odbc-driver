mod test_generator_util;

use cstr::WideChar;
use definitions::{
    CDataType, Desc, HDbc, HStmt, Handle, HandleType, SmallInt, SqlReturn, USmallInt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value;
use std::ptr::null_mut;
use std::{fmt, fs, path::PathBuf};

use crate::common::{allocate_env, connect_with_conn_string};
use crate::{
    common::{allocate_statement, get_sql_diagnostics, sql_return_to_string},
    test_runner::test_generator_util::generate_baseline_test_file,
};
use thiserror::Error;

const TEST_FILE_DIR: &str = "../resources/integration_test/tests";
const SQL_NULL_DATA: isize = -1;
const BUFFER_LENGTH: usize = 1000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("test runner failed for test {test}: expected {expected}, actual {actual}")]
    IntegrationTest {
        test: String,
        expected: String,
        actual: String,
        row: usize,
        column: usize,
    },
    #[error("mismatch in row counts for test {test}: expected {expected}, actual {actual}")]
    RowCount {
        test: String,
        expected: usize,
        actual: usize,
    },
    #[error("mismatch in result set column counts for test {test}: expected {expected}, actual {actual}, row {row}")]
    RSColumnCount {
        test: String,
        expected: usize,
        actual: usize,
        row: usize,
    },
    #[error("mismatch in metadata column counts for test {test}: expected {expected}, actual {actual}, descriptor {descriptor}")]
    MetadataColumnCount {
        test: String,
        expected: usize,
        actual: usize,
        descriptor: String,
    },
    #[error("mismatch in metadata column value for test {test}: expected {expected}, actual {actual}, descriptor {descriptor}, column {column}")]
    MetadataColumnValue {
        test: String,
        expected: String,
        actual: String,
        descriptor: String,
        column: usize,
    },
    #[error("failed to read file: {0}")]
    InvalidFile(String),
    #[error("failed to read directory: {0}")]
    InvalidDirectory(String),
    #[error("failed to load file paths: {0}")]
    InvalidFilePath(String),
    #[error("unable to deserialize YAML file: {0}")]
    CannotDeserializeYaml(String),
    #[error("not enough values in array expected: {0}, actual: {1}")]
    NotEnoughArguments(usize, usize),
    #[error("unsupported function {0}")]
    UnsupportedFunction(String),
    #[error("unexpected column metadata type in test input: {0}")]
    UnexpectedMetadataType(String),
    #[error("overflow caused by value {0}, err {1}")]
    ValueOverflowI16(i64, String),
    #[error("Function {0} failed with sql code {1}. Error message: {2}")]
    OdbcFunctionFailed(String, String, String),
    #[error("yaml err: {0}")]
    Yaml(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationTest {
    pub tests: Vec<TestEntry>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEntry {
    pub description: String,
    pub db: String,
    pub is_simple_type: Option<bool>,
    pub test_definition: TestDef,
    pub expected_result: Option<Vec<Vec<Value>>>,
    pub skip_reason: Option<String>,
    pub ordered: Option<bool>,
    pub expected_catalog_name: Option<Vec<Value>>,
    pub expected_case_sensitive: Option<Vec<Value>>,
    pub expected_column_name: Option<Vec<Value>>,
    pub expected_display_size: Option<Vec<Value>>,
    pub expected_length: Option<Vec<Value>>,
    pub expected_is_searchable: Option<Vec<Value>>,
    pub expected_is_unsigned: Option<Vec<Value>>,
    pub expected_sql_type: Option<Vec<Value>>,
    pub expected_bson_type: Option<Vec<Value>>,
    pub expected_precision: Option<Vec<Value>>,
    pub expected_scale: Option<Vec<Value>>,
    pub expected_nullability: Option<Vec<Value>>,
}

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TestDef {
    Query(String),
    Function(Vec<Value>),
}

impl fmt::Display for TestDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// resultset_tests runs the query and function tests contained in the TEST_FILE_DIR directory
#[test]
#[ignore]
pub fn resultset_tests() -> Result<()> {
    run_resultset_tests(false)
}

/// Run an integration test. The generate argument indicates whether
/// the test results should written to a file for baseline test file
/// generation, or be asserted for correctness.
pub fn run_resultset_tests(generate: bool) -> Result<()> {
    let env = allocate_env().unwrap();
    let paths = load_file_paths(PathBuf::from(TEST_FILE_DIR)).unwrap();
    for path in paths {
        let yaml = parse_test_file_yaml(&path).unwrap();

        for test in yaml.tests {
            match test.skip_reason {
                Some(sr) => println!("Skip Reason: {sr}"),
                None => {
                    let mut conn_str = crate::common::generate_default_connection_str();
                    conn_str.push_str(&("DATABASE=".to_owned() + &test.db + ";"));
                    if let Some(true) = test.is_simple_type {
                        conn_str.push_str("SIMPLE_TYPES_ONLY=1;");
                    }
                    let conn_handle = connect_with_conn_string(env, conn_str).unwrap();
                    let test_result = match test.test_definition {
                        TestDef::Query(ref q) => run_query_test(q, &test, conn_handle, generate),
                        TestDef::Function(ref f) => {
                            run_function_test(f, &test, conn_handle, generate)
                        }
                    };
                    assert_eq!(Ok(()), test_result);
                }
            }
        }
    }
    Ok(())
}

/// load_file_paths reads the given directory and returns a list of its file
/// path names.
pub fn load_file_paths(dir: PathBuf) -> Result<Vec<String>> {
    let mut paths: Vec<String> = vec![];
    let entries = fs::read_dir(dir).map_err(|e| Error::InvalidDirectory(format!("{e:?}")))?;
    for entry in entries {
        match entry {
            Ok(de) => {
                let path = de.path();
                if (path.extension().unwrap() == "yml") || (path.extension().unwrap() == "yaml") {
                    paths.push(path.to_str().unwrap().to_string());
                }
            }
            Err(e) => return Err(Error::InvalidFilePath(format!("{e:?}"))),
        };
    }
    Ok(paths)
}

/// parse_test_file_yaml deserializes the given YAML file into a
/// IntegrationTest struct.
pub fn parse_test_file_yaml(path: &str) -> Result<IntegrationTest> {
    let f = fs::File::open(path).map_err(|e| Error::InvalidFile(format!("{e:?}")))?;
    let integration_test: IntegrationTest =
        serde_yaml::from_reader(f).map_err(|e| Error::CannotDeserializeYaml(format!("{e:?}")))?;
    Ok(integration_test)
}

/// str_or_null converts value to a narrow string or null_mut() if null
fn str_or_null(value: &Value) -> *const u8 {
    if value.is_null() {
        null_mut()
    } else {
        match value.as_str() {
            Some(s) => s.as_ptr(),
            None => null_mut(),
        }
    }
}

/// wstr_or_null converts value to a wide string or null_mut() if null
/// Ok, it looks bizarre that we return the Vec here. This is to ensure that it lives as long
/// as the ptr.
fn wstr_or_null(value: &Value) -> (*const WideChar, Vec<WideChar>) {
    if value.is_null() {
        (null_mut(), Vec::new())
    } else {
        to_wstr_ptr(value.as_str().expect("Unable to cast value as string"))
    }
}

/// to_wstr_ptr converts a &str into a *const u16.
/// Ok, it looks bizarre that we return the Vec here. This is to ensure that it lives as long
/// as the ptr.
fn to_wstr_ptr(string: &str) -> (*const WideChar, Vec<WideChar>) {
    let mut v = cstr::to_widechar_vec(string);
    v.push(0);
    (v.as_ptr(), v)
}

fn to_i16(value: &Value) -> Result<i16> {
    let val = value.as_i64().expect("Unable to cast value as i64");
    i16::try_from(val).map_err(|e| Error::ValueOverflowI16(val, e.to_string()))
}

fn check_array_length(array: &Vec<Value>, length: usize) -> Result<()> {
    if array.len() < length {
        return Err(Error::NotEnoughArguments(length, array.len()));
    }
    Ok(())
}

/// Run a query integration test. The generate argument indicates
/// whether the test results should written to a file for baseline
/// test file generation, or be asserted for correctness.
fn run_query_test(query: &str, entry: &TestEntry, conn: HDbc, generate: bool) -> Result<()> {
    unsafe {
        let stmt: HStmt = allocate_statement(conn).unwrap();

        match definitions::SQLExecDirectW(stmt as HStmt, to_wstr_ptr(query).0, query.len() as i32) {
            SqlReturn::SUCCESS => {
                if generate {
                    generate_baseline_test_file(entry, stmt)
                } else {
                    validate_result_set(entry, stmt)
                }
            }
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLExecDirectW".to_string(),
                sql_return_to_string(sql_return),
                get_sql_diagnostics(HandleType::Stmt, stmt as Handle),
            )),
        }
    }
}

/// Run a function integration test. The generate argument indicates
/// whether the test results should written to a file for baseline
/// test file generation, or be asserted for correctness.
fn run_function_test(
    function: &Vec<Value>,
    entry: &TestEntry,
    conn: HDbc,
    generate: bool,
) -> Result<()> {
    let statement = allocate_statement(conn).unwrap();
    check_array_length(function, 1)?;
    let function_name = function[0].as_str().unwrap().to_lowercase();
    let sql_return = match function_name.as_str() {
        "sqlgettypeinfo" => {
            check_array_length(function, 2)?;
            unsafe {
                let data_type: definitions::SqlDataType =
                    std::mem::transmute(function[1].as_i64().unwrap() as i16);
                Ok(definitions::SQLGetTypeInfo(statement as HStmt, data_type))
            }
        }
        "sqltables" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(definitions::SQLTables(
                    statement as HStmt,
                    str_or_null(&function[1]),
                    to_i16(&function[2])?,
                    str_or_null(&function[3]),
                    to_i16(&function[4])?,
                    str_or_null(&function[5]),
                    to_i16(&function[6])?,
                    str_or_null(&function[7]),
                    to_i16(&function[8])?,
                ))
            }
        }
        "sqltablesw" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(definitions::SQLTablesW(
                    statement as HStmt,
                    wstr_or_null(&function[1]).0,
                    to_i16(&function[2])?,
                    wstr_or_null(&function[3]).0,
                    to_i16(&function[4])?,
                    wstr_or_null(&function[5]).0,
                    to_i16(&function[6])?,
                    wstr_or_null(&function[7]).0,
                    to_i16(&function[8])?,
                ))
            }
        }
        "sqlcolumns" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(definitions::SQLColumns(
                    statement as HStmt,
                    str_or_null(&function[1]),
                    to_i16(&function[2])?,
                    str_or_null(&function[3]),
                    to_i16(&function[4])?,
                    str_or_null(&function[5]),
                    to_i16(&function[6])?,
                    str_or_null(&function[7]),
                    to_i16(&function[8])?,
                ))
            }
        }
        "sqlcolumnsw" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(definitions::SQLColumnsW(
                    statement as HStmt,
                    wstr_or_null(&function[1]).0,
                    to_i16(&function[2])?,
                    wstr_or_null(&function[3]).0,
                    to_i16(&function[4])?,
                    wstr_or_null(&function[5]).0,
                    to_i16(&function[6])?,
                    wstr_or_null(&function[7]).0,
                    to_i16(&function[8])?,
                ))
            }
        }
        "sqlforeignkeysw" => {
            check_array_length(function, 13)?;
            unsafe {
                Ok(definitions::SQLForeignKeysW(
                    statement as HStmt,
                    wstr_or_null(&function[1]).0,
                    to_i16(&function[2])?,
                    wstr_or_null(&function[3]).0,
                    to_i16(&function[4])?,
                    wstr_or_null(&function[5]).0,
                    to_i16(&function[6])?,
                    wstr_or_null(&function[7]).0,
                    to_i16(&function[8])?,
                    wstr_or_null(&function[7]).0,
                    to_i16(&function[8])?,
                    wstr_or_null(&function[9]).0,
                    to_i16(&function[10])?,
                ))
            }
        }
        /*
        // SQL-1015: Investigate how to test missing functions from odbc-sys
        // The following functions are not implemented in odbc-sys

        "sqlprimarykeys" => {
            unsafe {
                Ok(definitions::SQLPrimaryKeys(
                    statement as HStmt,
                    str_or_null(&function[1]),
                    to_i16(&function[2]),
                    str_or_null(&function[3]),
                    to_i16(&function[4]),
                    str_or_null(&function[5]),
                    to_i16(&function[6]),
                ))
            }
        }
        "sqlspecialcolumns" => {
            unsafe {
                Ok(definitions::SQLSpecialColumns(
                    statement as HStmt,
                    to_i16(&function[1]),
                    str_or_null(&function[2]),
                    to_i16(&function[3]),
                    str_or_null(&function[4]),
                    to_i16(&function[5]),
                    str_or_null(&function[6]),
                    to_i16(&function[7]),
                    to_i16(&function[8]),
                    to_i16(&function[9]),
                ))
            }
        }
        "sqlstatistics" => {
            unsafe {
                Ok(definitions::SQLStatistics(
                    statement as HStmt,
                    str_or_null(&function[1]),
                    to_i16(&function[2]),
                    str_or_null(&function[3]),
                    to_i16(&function[4]),
                    str_or_null(&function[5]),
                    to_i16(&function[6]),
                    to_i16(&function[7]),
                    to_i16(&function[8]),
                ))
            }
        }
         */
        _ => Err(Error::UnsupportedFunction(function_name.clone())),
    };

    let sql_return_val = sql_return.unwrap();
    if sql_return_val != SqlReturn::SUCCESS {
        return Err(Error::OdbcFunctionFailed(
            function_name,
            sql_return_to_string(sql_return_val),
            get_sql_diagnostics(HandleType::Stmt, statement as *mut _),
        ));
    }
    if generate {
        generate_baseline_test_file(entry, statement)
    } else {
        validate_result_set(entry, statement)
    }
}

fn validate_result_set(entry: &TestEntry, stmt: HStmt) -> Result<()> {
    let column_count = get_column_count(stmt)?;
    let mut row_counter = 0;
    if let Some(expected_result) = entry.expected_result.as_ref() {
        let mut row_num = 0;
        while fetch_row(stmt)? {
            let expected_row_check = expected_result.get(row_counter);
            // If there are no more expected rows, continue fetching to get actual row count
            if let Some(expected_row) = expected_row_check {
                if expected_row.len() != column_count {
                    return Err(Error::RSColumnCount {
                        test: entry.description.clone(),
                        expected: expected_row.len(),
                        actual: column_count,
                        row: row_counter,
                    });
                }
                row_num += 1;
                for i in 0..(column_count) {
                    let expected_field = expected_row.get(i).unwrap();
                    let expected_data_type = if expected_field.is_number() {
                        match expected_field.is_f64() {
                            true => CDataType::SQL_C_DOUBLE,
                            false => CDataType::SQL_C_SLONG,
                        }
                    } else if expected_field.is_boolean() {
                        CDataType::SQL_C_BIT
                    } else {
                        CDataType::SQL_C_CHAR
                    };
                    let actual_field = get_data(stmt, i as USmallInt, expected_data_type)?;

                    if *expected_field != actual_field {
                        return Err(Error::IntegrationTest {
                            test: entry.description.clone(),
                            expected: expected_field.to_string(),
                            actual: actual_field.to_string(),
                            row: row_num,
                            column: i + 1,
                        });
                    }
                }
            }
            row_counter += 1;
        }
        if expected_result.len() != row_counter {
            return Err(Error::RowCount {
                test: entry.description.clone(),
                expected: expected_result.len(),
                actual: row_counter,
            });
        }
    }
    validate_result_set_metadata(entry, column_count, stmt)?;
    Ok(())
}

// Checks that column attributes match for the given descriptor type
fn validate_result_set_metadata_helper(
    stmt: HStmt,
    column_count: usize,
    description: String,
    descriptor: Desc,
    expected_metadata: &Option<Vec<Value>>,
) -> Result<()> {
    if let Some(exp_metadata) = &expected_metadata {
        if column_count != exp_metadata.len() {
            return Err(Error::MetadataColumnCount {
                test: description,
                expected: exp_metadata.len(),
                actual: column_count,
                descriptor: format!("{descriptor:?}"),
            });
        }
        for (i, current_exp_metadata) in exp_metadata.iter().enumerate().take(column_count) {
            // Columns start at 1, the column_count parameter is 0-indexed
            let actual_value = get_column_attribute(stmt, i + 1, descriptor, current_exp_metadata)?;
            match &current_exp_metadata {
                Value::Number(n) => {
                    if actual_value.as_i64() != Some(n.as_i64().unwrap()) {
                        return Err(Error::MetadataColumnValue {
                            test: description,
                            expected: n.to_string(),
                            actual: actual_value.to_string(),
                            descriptor: format!("{descriptor:?}"),
                            column: i,
                        });
                    }
                }
                Value::String(s) => {
                    if actual_value.as_str() != Some(s) {
                        return Err(Error::MetadataColumnValue {
                            test: description,
                            expected: s.to_string(),
                            actual: actual_value.to_string(),
                            descriptor: format!("{descriptor:?}"),
                            column: i,
                        });
                    }
                }
                meta_type => return Err(Error::UnexpectedMetadataType(format!("{meta_type:?}"))),
            }
        }
    }
    Ok(())
}

fn validate_result_set_metadata(entry: &TestEntry, column_count: usize, stmt: HStmt) -> Result<()> {
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::CatalogName,
        &entry.expected_catalog_name,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::CaseSensitive,
        &entry.expected_case_sensitive,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::DisplaySize,
        &entry.expected_display_size,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Length,
        &entry.expected_length,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Name,
        &entry.expected_column_name,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Searchable,
        &entry.expected_is_searchable,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Unsigned,
        &entry.expected_is_unsigned,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Type,
        &entry.expected_sql_type,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::TypeName,
        &entry.expected_bson_type,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Precision,
        &entry.expected_precision,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Scale,
        &entry.expected_scale,
    )?;
    validate_result_set_metadata_helper(
        stmt,
        column_count,
        entry.description.clone(),
        Desc::Nullable,
        &entry.expected_nullability,
    )?;
    Ok(())
}

fn get_column_attribute(
    stmt: HStmt,
    column: usize,
    field_identifier: Desc,
    column_metadata_type: &Value,
) -> Result<Value> {
    let string_length_ptr = &mut 0;
    let character_attrib_ptr: *mut std::ffi::c_void =
        Box::into_raw(Box::new([0; BUFFER_LENGTH])) as *mut _;
    let numeric_attrib_ptr = &mut 0;
    let result = unsafe {
        match definitions::SQLColAttributeW(
            stmt as *mut _,
            column as USmallInt,
            field_identifier,
            character_attrib_ptr,
            BUFFER_LENGTH as SmallInt,
            string_length_ptr,
            numeric_attrib_ptr,
        ) {
            SqlReturn::SUCCESS => Ok(match column_metadata_type {
                Value::String(_) => json!((cstr::from_widechar_ref_lossy(
                    &*(character_attrib_ptr as *const [WideChar; BUFFER_LENGTH])
                ))[0..(*string_length_ptr as usize / std::mem::size_of::<WideChar>())]
                    .to_string()),
                Value::Number(_) => json!(*numeric_attrib_ptr),
                meta_type => return Err(Error::UnexpectedMetadataType(format!("{meta_type:?}"))),
            }),
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLColAttributeW".to_string(),
                sql_return_to_string(sql_return),
                get_sql_diagnostics(HandleType::Stmt, stmt as *mut _),
            )),
        }
    };
    unsafe {
        let _ = Box::from_raw(character_attrib_ptr as *mut [u8; BUFFER_LENGTH]);
    }
    result
}

fn get_data(stmt: HStmt, column: USmallInt, data_type: CDataType) -> Result<Value> {
    let out_len_or_ind = &mut 0;
    let buffer: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; BUFFER_LENGTH])) as *mut _;
    let mut data: Value = Default::default();
    let result = unsafe {
        match definitions::SQLGetData(
            stmt as *mut _,
            // Result set columns start at 1, the column input parameter is 0-indexed
            column + 1,
            data_type,
            buffer as *mut _,
            BUFFER_LENGTH as isize,
            out_len_or_ind,
        ) {
            SqlReturn::SUCCESS | SqlReturn::NO_DATA => {
                if *out_len_or_ind == SQL_NULL_DATA {
                    data = json!(null);
                } else {
                    match data_type {
                        CDataType::SQL_C_CHAR => {
                            data = json!((String::from_utf8_lossy(&*(buffer as *const [u8; 256])))
                                [0..*out_len_or_ind as usize]
                                .to_string());
                        }
                        CDataType::SQL_C_SLONG => data = json!(*(buffer as *const i32)),
                        CDataType::SQL_C_DOUBLE => data = json!(*(buffer as *const f64)),
                        CDataType::SQL_C_BIT => data = json!(*(buffer as *const bool)),
                        _ => {}
                    };
                }
                Ok(data)
            }
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLGetData".to_string(),
                sql_return_to_string(sql_return),
                get_sql_diagnostics(HandleType::Stmt, stmt as *mut _),
            )),
        }
    };
    unsafe {
        let _ = Box::from_raw(buffer as *mut [u8; BUFFER_LENGTH]);
    }
    result
}

fn get_column_count(stmt: HStmt) -> Result<usize> {
    unsafe {
        let columns = &mut 0;
        match definitions::SQLNumResultCols(stmt as HStmt, columns) {
            SqlReturn::SUCCESS => Ok(*columns as usize),
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLNumResultCols".to_string(),
                sql_return_to_string(sql_return),
                get_sql_diagnostics(HandleType::Stmt, stmt as *mut _),
            )),
        }
    }
}

fn fetch_row(stmt: HStmt) -> Result<bool> {
    unsafe {
        match definitions::SQLFetch(stmt as HStmt) {
            SqlReturn::SUCCESS | SqlReturn::SUCCESS_WITH_INFO => Ok(true),
            SqlReturn::NO_DATA => Ok(false),
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLFetch".to_string(),
                sql_return_to_string(sql_return),
                get_sql_diagnostics(HandleType::Stmt, stmt as *mut _),
            )),
        }
    }
}
