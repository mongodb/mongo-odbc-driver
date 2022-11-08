use odbc::{create_environment_v3, Allocated, Connection, Handle, NoResult, Statement};
use odbc_sys::{CDataType, HStmt, SqlReturn, USmallInt};

use odbc::safe::AutocommitOn;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::ptr::null_mut;
use std::{fmt, fs, path::PathBuf};

use thiserror::Error;

const TEST_FILE_DIR: &str = "../resources/integration_test/tests";
const SQL_NULL_DATA: isize = -1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("test runner failed for test {test}: expected {expected:?}, actual {actual:?}")]
    IntegrationTest {
        test: String,
        expected: String,
        actual: String,
    },
    #[error("mismatch in row counts for test {test}: expected {expected:?}, actual {actual:?}")]
    RowCount {
        test: String,
        expected: usize,
        actual: usize,
    },
    #[error("mismatch in column counts for test {test}: expected {expected:?}, actual {actual:?}")]
    ColumnCount {
        test: String,
        expected: usize,
        actual: usize,
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
    #[error("overflow caused by value {0}, err {1}")]
    ValueOverflowI16(i64, String),
    #[error("function {0} failed with sql code {1}")]
    OdbcFunctionFailed(String, String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationTest {
    pub tests: Vec<TestEntry>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEntry {
    pub description: String,
    pub db: String,
    pub test_definition: TestDef,
    pub expected_result: Option<Vec<Vec<String>>>,
    pub skip_reason: Option<String>,
    pub ordered: Option<bool>,
    pub expected_column_name: Option<Vec<String>>,
    pub expected_sql_type: Option<Vec<String>>,
    pub expected_bson_type: Option<Vec<String>>,
    pub expected_precision: Option<Vec<i32>>,
    pub expected_scale: Option<Vec<i32>>,
    pub expected_nullability: Option<Vec<String>>,
}

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TestDef {
    Query(String),
    Function(Vec<Value>),
}

impl fmt::Display for TestDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// load_file_paths reads the given directory and returns a list its file path
/// names.
pub fn load_file_paths(dir: PathBuf) -> Result<Vec<String>, Error> {
    let mut paths: Vec<String> = vec![];
    let entries = fs::read_dir(dir).map_err(|e| Error::InvalidDirectory(format!("{:?}", e)))?;
    for entry in entries {
        match entry {
            Ok(de) => {
                let path = de.path();
                if (path.extension().unwrap() == "yml") || (path.extension().unwrap() == "yaml") {
                    paths.push(path.to_str().unwrap().to_string());
                }
            }
            Err(e) => return Err(Error::InvalidFilePath(format!("{:?}", e))),
        };
    }
    Ok(paths)
}

/// parse_test_file_yaml deserializes the given YAML file into a
/// IntegrationTest struct.
pub fn parse_test_file_yaml(path: &str) -> Result<IntegrationTest, Error> {
    let f = fs::File::open(path).map_err(|e| Error::InvalidFile(format!("{:?}", e)))?;
    let integration_test: IntegrationTest =
        serde_yaml::from_reader(f).map_err(|e| Error::CannotDeserializeYaml(format!("{:?}", e)))?;
    Ok(integration_test)
}

/// integration_test runs the query and function tests contained in the TEST_FILE_DIR directory
#[test]
#[ignore]
pub fn integration_test() -> Result<(), Error> {
    let env = create_environment_v3().unwrap();
    let paths = load_file_paths(PathBuf::from(TEST_FILE_DIR)).unwrap();
    for path in paths {
        let yaml = parse_test_file_yaml(&path).unwrap();

        for test in yaml.tests {
            match test.skip_reason {
                Some(sr) => println!("Skip Reason: {}", sr),
                None => {
                    let mut conn_str = crate::common::generate_default_connection_str();
                    conn_str.push_str(&(";DATABASE=".to_owned() + &test.db));
                    let connection = env
                        .connect_with_connection_string(conn_str.as_str())
                        .unwrap();
                    let test_result = match test.test_definition {
                        TestDef::Query(ref q) => run_query_test(q, &test, &connection),
                        TestDef::Function(ref f) => run_function_test(f, &test, &connection),
                    };
                    drop(connection);
                    assert_eq!(Ok(()), test_result);
                }
            }
        }
    }
    Ok(())
}

/// str_or_null converts value to a narrow string or null_mut() if null
fn str_or_null(value: &Value) -> *const u8 {
    if value.is_null() {
        null_mut()
    } else {
        odbc_api::handles::SqlText::new(value.as_str().expect("Unable to cast value as string"))
            .ptr()
    }
}

/// wstr_or_null converts value to a wide string or null_mut() if null
fn wstr_or_null(value: &Value) -> *const u16 {
    if value.is_null() {
        null_mut()
    } else {
        to_wstr_ptr(value.as_str().expect("Unable to cast value as string"))
    }
}

fn to_wstr_ptr(string: &str) -> *const u16 {
    let mut v: Vec<u16> = string.encode_utf16().collect();
    v.push(0);
    v.as_ptr()
}

fn to_i16(value: &Value) -> Result<i16, Error> {
    let val = value.as_i64().expect("Unable to cast value as i64");
    i16::try_from(val).map_err(|e| Error::ValueOverflowI16(val, e.to_string()))
}

fn check_array_length(array: &Vec<Value>, length: usize) -> Result<(), Error> {
    if array.len() < length {
        return Err(Error::NotEnoughArguments(length, array.len()));
    }
    Ok(())
}

fn run_query_test(
    query: &str,
    entry: &TestEntry,
    conn: &Connection<AutocommitOn>,
) -> Result<(), Error> {
    let stmt = Statement::with_parent(conn).unwrap();
    unsafe {
        match odbc_sys::SQLExecDirectW(
            stmt.handle() as *mut _,
            to_wstr_ptr(query),
            query.len() as i32,
        ) {
            SqlReturn::SUCCESS => {}
            sql_return => {
                return Err(Error::OdbcFunctionFailed(
                    "SQLExecDirectW".to_string(),
                    format!("{:?}", sql_return),
                ))
            }
        }
        validate_result_set(entry, stmt)
    }
}

fn run_function_test(
    function: &Vec<Value>,
    entry: &TestEntry,
    conn: &Connection<AutocommitOn>,
) -> Result<(), Error> {
    let statement = Statement::with_parent(conn).unwrap();
    check_array_length(function, 1)?;
    let function_name = function[0].as_str().unwrap().to_lowercase();
    let sql_return = match function_name.as_str() {
        "sqlgettypeinfo" => {
            check_array_length(function, 2)?;
            unsafe {
                let data_type: odbc_sys::SqlDataType =
                    std::mem::transmute(function[1].as_i64().unwrap() as i16);
                Ok(odbc_sys::SQLGetTypeInfo(
                    statement.handle() as HStmt,
                    data_type,
                ))
            }
        }
        "sqltables" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(odbc_sys::SQLTables(
                    statement.handle() as HStmt,
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
                Ok(odbc_sys::SQLTablesW(
                    statement.handle() as HStmt,
                    wstr_or_null(&function[1]),
                    to_i16(&function[2])?,
                    wstr_or_null(&function[3]),
                    to_i16(&function[4])?,
                    wstr_or_null(&function[5]),
                    to_i16(&function[6])?,
                    wstr_or_null(&function[7]),
                    to_i16(&function[8])?,
                ))
            }
        }
        "sqlforeignkeysw" => {
            check_array_length(function, 13)?;
            unsafe {
                Ok(odbc_sys::SQLForeignKeysW(
                    statement.handle() as HStmt,
                    wstr_or_null(&function[1]),
                    to_i16(&function[2])?,
                    wstr_or_null(&function[3]),
                    to_i16(&function[4])?,
                    wstr_or_null(&function[5]),
                    to_i16(&function[6])?,
                    wstr_or_null(&function[7]),
                    to_i16(&function[8])?,
                    wstr_or_null(&function[7]),
                    to_i16(&function[8])?,
                    wstr_or_null(&function[7]),
                    to_i16(&function[8])?,
                ))
            }
        }
        /*
        // SQL-1015: Investigate how to test missing functions from odbc-sys
        // The following functions are not implemented in odbc-sys

        "sqlprimarykeys" => {
            unsafe {
                Ok(odbc_sys::SQLPrimaryKeys(
                    statement.handle() as HStmt,
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
                Ok(odbc_sys::SQLSpecialColumns(
                    statement.handle() as HStmt,
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
                Ok(odbc_sys::SQLStatistics(
                    statement.handle() as HStmt,
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
        "sqltableprivileges" => {
            unsafe {
                Ok(odbc_sys::SQLTablePrivileges(
                    statement.handle() as HStmt,
                    str_or_null(&function[1]),
                    to_i16(&function[2]),
                    str_or_null(&function[3]),
                    to_i16(&function[4]),
                    str_or_null(&function[5]),
                    to_i16(&function[6]),
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
            format!("{:?}", sql_return_val),
        ));
    }
    validate_result_set(entry, statement)
}

fn validate_result_set(
    entry: &TestEntry,
    stmt: Statement<Allocated, NoResult, AutocommitOn>,
) -> Result<(), Error> {
    let columns = get_column_count(&stmt)?;
    let mut row_counter = 0;
    if let Some(expected_result) = entry.expected_result.as_ref() {
        while fetch_row(&stmt)? {
            let expected_row_check = expected_result.get(row_counter);
            // If there are no more expected rows, continue fetching to get actual row count
            if let Some(expected_row) = expected_row_check {
                if expected_row.len() != columns {
                    return Err(Error::ColumnCount {
                        test: entry.description.clone(),
                        expected: expected_row.len(),
                        actual: columns,
                    });
                }

                for i in 1..(columns + 1) {
                    let expected_field = expected_row.get(i - 1);
                    let data = get_data_as_string(&stmt, i as USmallInt)?;
                    if *expected_field.unwrap() != data {
                        return Err(Error::IntegrationTest {
                            test: entry.description.clone(),
                            expected: (*expected_field.unwrap().to_string()).parse().unwrap(),
                            actual: data,
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
    Ok(())
}

fn get_data_as_string(
    stmt: &Statement<Allocated, NoResult, AutocommitOn>,
    column: USmallInt,
) -> Result<String, Error> {
    const BUFFER_LENGTH: usize = 200;
    let out_len_or_ind = &mut 0;
    let char_buffer: *mut std::ffi::c_void =
        Box::into_raw(Box::new([0u8; BUFFER_LENGTH])) as *mut _;
    let data: String;
    unsafe {
        match odbc_sys::SQLGetData(
            stmt.handle() as *mut _,
            column,
            CDataType::Char,
            char_buffer as *mut _,
            BUFFER_LENGTH as isize,
            out_len_or_ind,
        ) {
            SqlReturn::SUCCESS => {}
            sql_return => {
                return Err(Error::OdbcFunctionFailed(
                    "SQLGetData".to_string(),
                    format!("{:?}", sql_return),
                ))
            }
        }

        if *out_len_or_ind == SQL_NULL_DATA {
            // Mapping `null` data type to string "NULL"
            // Make sure not to have "NULL" string values in dataset
            data = "NULL".to_string();
        } else {
            data = (String::from_utf8_lossy(&*(char_buffer as *const [u8; 256])))
                [0..*out_len_or_ind as usize]
                .to_string();
        }
    }
    Ok(data)
}

fn get_column_count(stmt: &Statement<Allocated, NoResult, AutocommitOn>) -> Result<usize, Error> {
    unsafe {
        let columns = &mut 0;
        match odbc_sys::SQLNumResultCols(stmt.handle() as HStmt, columns) {
            SqlReturn::SUCCESS => {}
            sql_return => {
                return Err(Error::OdbcFunctionFailed(
                    "SQLNumResultCols".to_string(),
                    format!("{:?}", sql_return),
                ))
            }
        }
        Ok(*columns as usize)
    }
}

fn fetch_row(stmt: &Statement<Allocated, NoResult, AutocommitOn>) -> Result<bool, Error> {
    unsafe {
        match odbc_sys::SQLFetch(stmt.handle() as HStmt) {
            SqlReturn::SUCCESS => Ok(true),
            SqlReturn::NO_DATA => Ok(false),
            sql_return => Err(Error::OdbcFunctionFailed(
                "SQLFetch".to_string(),
                format!("{:?}", sql_return),
            )),
        }
    }
}
