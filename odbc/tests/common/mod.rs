extern crate odbc_api;

use self::odbc_api::handles::StatementImpl;
use self::odbc_api::{Connection, Cursor, CursorImpl, Environment};
use lazy_static::lazy_static;
use odbc_api::ResultSetMetadata;
use std::env;

#[derive(Debug, PartialEq)]
pub struct ResultsetMetadata {
    pub expected_sql_types: Vec<String>,
    pub expected_bson_type: Vec<String>,
    pub expected_catalog_name: Vec<String>,
    pub expected_column_class_name: Vec<String>,
    pub expected_column_display_size: Vec<String>,
    pub expected_column_label: Vec<String>,
    pub expected_column_type: Vec<String>,
    pub expected_precision: Vec<String>,
    pub expected_scale: Vec<String>,
    pub expected_schema_name: Vec<String>,
    pub expected_is_auto_increment: Vec<String>,
    pub expected_is_case_sensitive: Vec<String>,
    pub expected_is_currency: Vec<String>,
    pub expected_is_definitely_writable: Vec<String>,
    pub expected_is_nullable: Vec<String>,
    pub expected_is_read_only: Vec<String>,
    pub expected_is_searchable: Vec<String>,
    pub expected_is_signed: Vec<String>,
    pub expected_is_writable: Vec<String>,
    pub expected_names: Vec<String>,
}

// Allocate a new environment handle.
// Most tests will only need one and this should be part of the setup mechanism.
lazy_static! {
    pub static ref ODBC_ENV: Environment = Environment::new().unwrap();
}

#[allow(dead_code)]
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

    let _dsn = env::var("ADL_TEST_DSN");
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

#[allow(dead_code)]
/// Validate the resultset and its metadata
pub fn validate_rs(
    expected_row_count_opt: Option<i64>,
    expected_result_set_opt: Option<&Vec<Vec<String>>>,
    expected_rs_meta_opt: Option<&ResultsetMetadata>,
    rs_cursor: &mut CursorImpl<StatementImpl>,
) {
    // Validate metadata
    if let Some(expected_rs_meta) = expected_rs_meta_opt {
        let num_col = rs_cursor.num_result_cols().unwrap();
        assert_eq!(expected_rs_meta.expected_names.len() as i16, num_col);
        let column_names = rs_cursor.column_names().unwrap();
        for (index, name) in column_names.enumerate() {
            assert_eq!(expected_rs_meta.expected_names[index], name.unwrap());
        }
    }

    if expected_row_count_opt.is_some() || expected_result_set_opt.is_some() {
        let mut row_count: i64 = 0;
        while let Ok(_row) = rs_cursor.next_row() {
            row_count += 1;
            // TODO - Validate RS content
        }

        // Validate row count
        if expected_row_count_opt.is_some() {
            assert_eq!(expected_row_count_opt.unwrap(), row_count)
        }
    }

    // TODO Do we need to clean-up? sqlCloseCursor? sqlFreeStatement?
    // The caller will have a clean-up session which call sqlDisconnect
    // and should free all the statements attached to the connection
}
