use lazy_static::lazy_static;
use odbc_api::{
    buffers::{BufferDescription, BufferKind, ColumnarAnyBuffer},
    handles::Statement,
    Connection, Cursor, CursorImpl, Environment, Nullability, RowSetCursor,
};
use odbc_sys::SqlReturn;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::ptr::null_mut;
use std::{collections::BTreeMap, env, fmt, fs, path::PathBuf};
use thiserror::Error;

const TEST_FILE_DIR: &str = "../resources/integration_test/tests";

lazy_static! {
    pub static ref ODBC_ENV: Environment = Environment::new().unwrap();
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /*
    #[error("test runner failed for test {test}: expected {expected:?}, actual {actual:?}")]
    IntegrationTest {
        test: String,
        expected: String,
        actual: String,
    },
     */
    #[error("failed to read file: {0}")]
    InvalidFile(String),
    #[error("failed to read directory: {0}")]
    InvalidDirectory(String),
    #[error("failed to load file paths: {0}")]
    InvalidFilePath(String),
    #[error("unable to deserialize YAML file: {0}")]
    CannotDeserializeYaml(String),
    #[error("unsuccessful SQL return code encountered: {0}")]
    SqlReturn(String),
    #[error("not enough values in array expected: {0}, actual: {1}")]
    NotEnoughArguments(usize, usize),
    #[error("unsupported function {0}")]
    UnsupportedFunction(String),
    #[error("overflow caused by value {0}, err {1}")]
    ValueOverflowI16(i64, String),
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
    pub expected_result: Option<Vec<BTreeMap<String, Value>>>,
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

/// connect obtains a connection to the local Atlas Data Federation instance and returns Connection
/// it fetches environment variables to create the connection string
fn connect() -> Result<Connection<'static>, Error> {
    let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
    let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
    let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");
    let auth_db = match env::var("ADF_TEST_LOCAL_AUTH_DB") {
        Ok(val) => val,
        Err(_) => "admin".to_string(), //Default auth db
    };
    let db = match env::var("ADF_TEST_LOCAL_DB") {
        Ok(val) => val,
        Err(_) => "integration_test".to_string(), //Default DB name
    };
    let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
        Ok(val) => val,
        Err(_) => "ADF_ODBC_DRIVER".to_string(), //Default driver name
    };

    let connection_string = format!(
        "Driver={};PWD={};USER={};SERVER={};AUTH_SRC={};Database={}",
        driver, password, user_name, host, auth_db, db
    );
    let connection = ODBC_ENV
        .connect_with_connection_string(&connection_string)
        .unwrap();

    assert!(!connection.is_dead().unwrap());
    Ok(connection)
}

/// integration_test runs the query and function tests contained in the TEST_FILE_DIR directory
#[test]
#[ignore]
pub fn integration_test() -> Result<(), Error> {
    let paths = load_file_paths(PathBuf::from(TEST_FILE_DIR)).unwrap();
    for path in paths {
        let yaml = parse_test_file_yaml(&path).unwrap();

        for test in yaml.tests {
            match test.skip_reason {
                Some(sr) => println!("Skip Reason: {}", sr),
                None => {
                    let connection = connect().unwrap();
                    let test_result = match test.test_definition {
                        TestDef::Query(ref q) => run_query_test(q, &test, &connection),
                        TestDef::Function(ref f) => run_function_test(f, &test, &connection),
                    };
                    drop(connection);
                    return test_result;
                }
            }
        }
    }
    Ok(())
}

fn run_query_test(query: &str, entry: &TestEntry, conn: &Connection) -> Result<(), Error> {
    let cursor = conn.execute(query, ()).unwrap().unwrap();
    validate_result_set(entry, cursor);
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

/// strw_or_null converts value to a wide string or null_mut() if null
fn strw_or_null(value: &Value) -> *const u16 {
    if value.is_null() {
        null_mut()
    } else {
        let mut v: Vec<u16> = value
            .as_str()
            .expect("Unable to cast value as string")
            .encode_utf16()
            .collect();
        v.push(0);
        v.as_ptr()
    }
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

fn run_function_test(
    function: &Vec<Value>,
    entry: &TestEntry,
    conn: &Connection,
) -> Result<(), Error> {
    let preallocated = conn.preallocate().unwrap();
    let statement = preallocated.into_statement();
    check_array_length(function, 1)?;
    let function_name = function[0].as_str().unwrap().to_lowercase();
    let sql_return = match function_name.as_str() {
        "sqlgettypeinfo" => {
            check_array_length(function, 2)?;
            unsafe {
                let data_type: odbc_sys::SqlDataType =
                    std::mem::transmute(function[1].as_i64().unwrap() as i16);
                Ok(odbc_sys::SQLGetTypeInfo(statement.as_sys(), data_type))
            }
        }
        "sqltables" => {
            check_array_length(function, 9)?;
            unsafe {
                Ok(odbc_sys::SQLTables(
                    statement.as_sys(),
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
        "sqlforeignkeysw" => {
            check_array_length(function, 13)?;
            unsafe {
                Ok(odbc_sys::SQLForeignKeysW(
                    statement.as_sys(),
                    strw_or_null(&function[1]),
                    to_i16(&function[2])?,
                    strw_or_null(&function[3]),
                    to_i16(&function[4])?,
                    strw_or_null(&function[5]),
                    to_i16(&function[6])?,
                    strw_or_null(&function[7]),
                    to_i16(&function[8])?,
                    strw_or_null(&function[7]),
                    to_i16(&function[8])?,
                    strw_or_null(&function[7]),
                    to_i16(&function[8])?,
                ))
            }
        }
        /*
        // The following functions are not implemented in odbc-sys

        "sqlprimarykeys" => {
            unsafe {
                Ok(odbc_sys::SQLPrimaryKeys(
                    statement.as_sys(),
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
                    statement.as_sys(),
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
                    statement.as_sys(),
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
                    statement.as_sys(),
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
        _ => Err(Error::UnsupportedFunction(function_name)),
    };

    let sql_return_val = sql_return.unwrap();
    if sql_return_val != SqlReturn::SUCCESS {
        return Err(Error::SqlReturn(format!("{:?}", sql_return_val)));
    }
    unsafe {
        let cursor = CursorImpl::new(statement);
        validate_result_set(entry, cursor);
    }

    Ok(())
}

/// allocate_buffer takes the cursor and allocates a buffer based on the types of the columns
pub fn allocate_buffer(mut cursor: impl Cursor) -> RowSetCursor<impl Cursor, ColumnarAnyBuffer> {
    let mut column_description = Default::default();
    let buffer_description: Vec<_> = (0..cursor.num_result_cols().unwrap())
        .map(|index| {
            cursor
                .describe_col(index as u16 + 1, &mut column_description)
                .unwrap();
            Ok(BufferDescription {
                nullable: matches!(
                    column_description.nullability,
                    Nullability::Unknown | Nullability::Nullable
                ),
                // Use reasonable sized text, in case we do not know the buffer type.
                kind: BufferKind::from_data_type(column_description.data_type)
                    .unwrap_or(BufferKind::Text { max_str_len: 255 }),
            })
        })
        .collect::<Result<_, Error>>()
        .unwrap();
    let buffer = ColumnarAnyBuffer::from_description(5000, buffer_description.into_iter());
    cursor.bind_buffer(buffer).unwrap()
}

fn validate_result_set(_entry: &TestEntry, cursor: impl Cursor) {
    let _row_set_cursor = allocate_buffer(cursor);
    // Compare expected to actual rowset
    // Implement as part of SQL-984
}
