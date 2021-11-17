extern crate odbc_api;
use lazy_static::lazy_static;
use odbc_api::*;
use std::env;

// Allocate a new environment handle.
// Most tests will only need one and this should be part of the setup mechanism.
lazy_static! {
    pub static ref ODBC_ENV: Environment = {
        let env = Environment::new().unwrap();
        env
    };
}

/// Connect to the given Driver or DSN with the provided uid, pwd and host.
/// The default auth db is 'admin' is not specified.
pub fn connect() -> Connection<'static> {
    let user_name = env::var("ADL_TEST_USER").expect("ADL_TEST_USER is not set");
    let password = env::var("ADL_TEST_PWD").expect("ADL_TEST_PWD is not set");
    let host = env::var("ADL_TEST_HOST").expect("ADL_TEST_HOST is not set");

    let auth_db = match env::var("ADL_TEST_AUTH_DB") {
        Ok(val) => val,
        Err(_e) => "admin".to_string(), //Default auth db
    };

    let db = env::var("ADL_TEST_DB");
    let driver = match env::var("ADL_TEST_DRIVER") {
        Ok(val) => val,
        Err(_e) => "ADL_ODBC_DRIVER".to_string(), //Default driver name
    };

    let dsn = env::var("ADL_TEST_DSN");
    // TODO : If DSN is specified, it should take over using 'DRIVER=' for connecting and use 'DSN='.

    let mut connection_string = format!(
        "Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}",
        driver, user_name, password, host, auth_db,
    );

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val)),
        Err(_e) => (), // Do nothing
    };

    ODBC_ENV
        .connect_with_connection_string(&connection_string)
        .unwrap()
}

/// Call sqlTables with the given arguments and validate the resultset.
pub fn sqltables(
    catalog_name: Option<&str>,
    schema_name: Option<&str>,
    table_name: Option<&str>,
    table_type: Option<&str>,
    expected_row_count: Option<i64>,
) {
    let conn = connect();
    let mut cursor = conn
        .tables(catalog_name, schema_name, table_name, table_type)
        .unwrap();
    let num_col = cursor.num_result_cols().unwrap();
    assert_eq!(5, num_col);
    let mut row_count: i64 = 0;
    while let Ok(row) = cursor.next_row() {
        row_count = row_count + 1;
        // TODO - Validate RS content
    }
    if expected_row_count.is_some() {
        assert_eq!(expected_row_count.unwrap(), row_count)
    }
}
