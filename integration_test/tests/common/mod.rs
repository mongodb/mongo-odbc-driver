use std::env;

/// Generate the default connection setting defined for the tests using a connection string
/// of the form 'Driver={};PWD={};USER={};SERVER={};AUTH_SRC={}'.
/// The default driver is 'ADF_ODBC_DRIVER' if not specified.
/// The default auth db is 'admin' if not specified.
pub fn generate_default_connection_str() -> String {
    let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
    let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
    let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");

    let auth_db = match env::var("ADF_TEST_LOCAL_AUTH_DB") {
        Ok(val) => val,
        Err(_e) => "admin".to_string(), //Default auth db
    };

    let db = env::var("ADF_TEST_LOCAL_DB");
    let driver = match env::var("ADF_TEST_LOCAL_DRIVER") {
        Ok(val) => val,
        Err(_e) => "ADF_ODBC_DRIVER".to_string(), //Default driver name
    };

    let mut connection_string = format!(
        "Driver={{{}}};USER={};PWD={};SERVER={};AUTH_SRC={};",
        driver, user_name, password, host, auth_db,
    );

    // If a db is specified add it to the connection string
    match db {
        Ok(val) => connection_string.push_str(&("DATABASE=".to_owned() + &val + ";")),
        Err(_e) => (), // Do nothing
    };

    connection_string
}
