use lazy_static::lazy_static;
use odbc_api::{Connection, Environment, Error};
use std::env;

// Allocate a new environment handle.
// Most tests will only need one and this should be part of the setup mechanism.
lazy_static! {
    pub static ref ODBC_ENV: Environment = Environment::new().unwrap();
}

/// Generate the default connection setting defined for the tests using a connection string
/// of the form 'Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}'.
/// The default driver is 'ADL_ODBC_DRIVER' if not specified.
/// The default auth db is 'admin' if not specified.
fn generate_default_connection_str() -> String {
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

    let mut connection_string = format!(
        "Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}",
        driver, user_name, password, host, auth_db,
    );

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val)),
        Err(_e) => (), // Do nothing
    };

    connection_string
}

/// Connect using the given connection string or the default settings if no connection string are provided.
pub fn connect(connection_string: Option<&str>) -> Result<Connection<'_>, Error> {
    match connection_string {
        Some(str) => ODBC_ENV.connect_with_connection_string(str),
        None => ODBC_ENV.connect_with_connection_string(generate_default_connection_str().as_str()),
    }
}
