mod common;

use common::{connect, generate_default_connection_str};

//#[test]
fn test_invalid_connection() {
    // Invalid Driver name
    let conn_str = "Driver=ADL_ODBC_DRIVER;PWD=N/A;USER=N/A;SERVER=N/A;AUTH_SRC=N/A";
    let result = connect(Some(conn_str));
    assert!(
        result.is_err(),
        "The connection should have failed, but it was successful."
    );
}

#[allow(dead_code)]
#[test]
// Uncomment to verify that driver is installed.
// It will still fail until SQLDriverConnect is implemented, but it will show that the DM found the driver.
fn test_default_connection() {
    connect(None).unwrap();
}
