use odbc_api::{Connection, Environment, Cursor, RowSetCursor,  Nullability, CursorImpl,
    buffers::{BufferDescription, BufferKind, ColumnarAnyBuffer},
    handles::Statement};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use thiserror::Error;
use odbc_sys::SqlReturn;
use std::{
    collections::{BTreeMap},
    fs,
    io::Read,
    path::PathBuf,
    env,
};
use std::ptr::null_mut;

const TEST_FILE_DIR: &str = "../resources/integration_test/tests";

lazy_static! {
   pub static ref ODBC_ENV: Environment =  {
       let env = Environment::new().unwrap();
       env
   };
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
    #[error("unable to read file to string: {0}")]
    CannotReadFileToString(String),
    #[error("failed to read file: {0}")]
    InvalidFile(String),
    #[error("failed to read directory: {0}")]
    InvalidDirectory(String),
    #[error("failed to load file paths: {0}")]
    InvalidFilePath(String),
    #[error("unable to deserialize YAML file: {0}")]
    CannotDeserializeYaml(String),
    #[error("missing query or function in test: {0}")]
    MissingQueryOrFunction(String),
    #[error("unsuccessful SQL return code encountered: {0}")]
    SqlReturnError(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegrationTest {
    pub tests: Vec<TestEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TestEntry {
    pub description: String,
    pub db: String,
    pub query: Option<String>,
    pub function: Option<Vec<Value>>,
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

/// load_file_paths reads the given directory and returns a list its file path
/// names.
pub fn load_file_paths(dir: PathBuf) -> Result<Vec<String>, Error> {
    let mut paths: Vec<String> = vec![];
    let entries = fs::read_dir(dir).map_err(|e| Error::InvalidDirectory(format!("{:?}", e)))?;
    for entry in entries {
        match entry {
            Ok(de) => {
                let path = de.path();
                if ( path.extension().unwrap() == "yml") || ( path.extension().unwrap() == "yaml" ) {
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
    let mut f = fs::File::open(path).map_err(|e| Error::InvalidFile(format!("{:?}", e)))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|e| Error::CannotReadFileToString(format!("{:?}", e)))?;
    let yaml: IntegrationTest = serde_yaml::from_str(&contents)
        .map_err(|e| Error::CannotDeserializeYaml(format!("{:?}", e)))?;
    Ok(yaml)
}

/// connect obtains a connection to the local Atlas Data Federation instance and returns Connection
/// it fetches environment variables to create the connection string
fn connect() -> Result<Connection<'static>, Error> {
    let user_name = env::var("ADL_TEST_USER").expect("ADL_TEST_USER is not set");
    let password = env::var("ADL_TEST_PWD").expect("ADL_TEST_PWD is not set");
    let host = env::var("ADL_TEST_HOST").expect("ADL_TEST_HOST is not set");
    let auth_db = match env::var("ADL_TEST_AUTH_DB"){
       Ok(val) => val,
       Err(_) => "admin".to_string(), //Default auth db
    };
    let db = match env::var("ADL_TEST_DB")
    {
       Ok(val) => val,
       Err(_) => "INTEGRATION_TEST".to_string(), //Default driver name
    };
    let driver = match env::var("ADL_TEST_DRIVER")
    {
       Ok(val) => val,
       Err(_) => "ADL_ODBC_DRIVER".to_string(), //Default driver name
    };

    let connection_string = format!(
       "Driver={{{}}};PWD={};UID={};SERVER={};AUTH_SRC={};Database={}",
       driver, password, user_name, host, auth_db, db);
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
                Some(_) => continue,
                None => {
                    let connection = connect().unwrap();
                    let test_result = match (&test.query, &test.function) {
                        (Some(_),_) => {
                            run_query_test(&test, &connection)
                        }
                        (None, Some(_)) => {
                            run_function_test(&test, &connection)
                        }
                        (_,_) => return Err(Error::MissingQueryOrFunction(format!("{:?}", test.description)))
                    };
                    drop(connection);
                    return test_result;
                }
            }
        }
    }
    Ok(())
}

fn run_query_test(entry : &TestEntry, conn : &Connection) -> Result<(), Error>{
    let cursor = conn.execute(&entry.query.as_ref().unwrap(), ()).unwrap().unwrap();
    validate_result_set(entry, cursor);
    Ok(())
}

fn str_or_null(value : &Value) -> *const u8 {
    if value.is_null() {
        null_mut()
    } else {
        odbc_api::handles::SqlText::new(&value.as_str().expect("Unable to cast value as string")).ptr()
    }
}

fn to_i16(value : &Value) -> i16 {
        value.as_i64().expect("Unable to cast value as i64") as i16
}

fn check_array_length(array : &Vec<Value>, length : usize) {
    if array.len() < length {
        panic!("not enough values in array expected: {}, actual: {}", length, array.len())
    }
}

fn run_function_test(entry : &TestEntry, conn : &Connection) -> Result<(), Error>{
    let function = entry.function.as_ref().unwrap();
    let preallocated = conn.preallocate().unwrap();
    let statement = preallocated.into_statement();
    check_array_length(function, 1);
    let function_name = function[0].as_str().unwrap().to_lowercase();
    let sql_return = match  function_name.as_str() {
        "sqlgettypeinfo" => {
            check_array_length(function, 2);
            unsafe {
                let data_type : odbc_sys::SqlDataType = std::mem::transmute(function[1].as_i64().unwrap() as i16);
                Ok(odbc_sys::SQLGetTypeInfo(
                    statement.as_sys(),
                    data_type,
                ))
             }
        }
        "sqltables" => {
            check_array_length(function, 9);
            unsafe {
                 Ok(odbc_sys::SQLTables(
                    statement.as_sys(),
                    str_or_null(&function[1]),
                    to_i16(&function[2]),
                    str_or_null(&function[3]),
                    to_i16(&function[4]),
                    str_or_null(&function[5]),
                    to_i16(&function[6]),
                    str_or_null(&function[7]),
                    to_i16(&function[8]),
                ))
            }
        }
        /*
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
        _ => Err(format!("unknown function {}", function_name))
    };

    let sql_return_val = sql_return.unwrap();
    if (sql_return_val != SqlReturn::SUCCESS) & (sql_return_val != SqlReturn::SUCCESS_WITH_INFO) {
        return Err(Error::SqlReturnError(format!("{:?}", sql_return_val)));
    }
    unsafe {
        let cursor = CursorImpl::new(statement);
        validate_result_set(entry, cursor);
    }

    Ok(())
}

/// allocate_buffer takes the cursor and allocates a buffer based on the types of the columns
pub fn allocate_buffer(mut cursor: impl Cursor ) ->  RowSetCursor<impl Cursor, ColumnarAnyBuffer> {
    let mut column_description = Default::default();
    let buffer_description : Vec<_> = (0..cursor.num_result_cols().unwrap()).map(|index| {
        cursor.describe_col(index as u16 + 1, &mut column_description).unwrap();
        Ok(BufferDescription {
            nullable: matches!(column_description.nullability, Nullability::Unknown | Nullability::Nullable),
            // Use reasonable sized text, in case we do not know the buffer type.
            kind: BufferKind::from_data_type(column_description.data_type)
                .unwrap_or(BufferKind::Text { max_str_len: 255 })
        })
    }).collect::<Result<_, Error>>().unwrap();
    let buffer = ColumnarAnyBuffer::from_description(5000, buffer_description.into_iter());
    let row_set_cursor = cursor.bind_buffer(buffer).unwrap();
    row_set_cursor
}

fn validate_result_set(_entry: &TestEntry, cursor: impl Cursor) {
    let _row_set_cursor = allocate_buffer(cursor);
    // Compare expected to actual rowset
    // Implement as part of SQL-984
}
