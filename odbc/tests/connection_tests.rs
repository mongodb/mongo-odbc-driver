mod common;

use common::connect;

//#[test]
//fn test_invalid_connection() {
//    // Invalid Driver name
//    let conn_str = "Driver=ADL_ODBC_DRIVER;PWD=N_A;USER=N_A;SERVER=N_A;AUTH_SRC=N_A";
//    let result = connect(Some(conn_str));
//    assert!(
//        result.is_err(),
//        "The connection should have failed, but it was successful."
//    );
//}

#[test]
// Uncomment to verify that driver is installed.
// It will still fail until SQLDriverConnect is implemented, but it will show that the DM found the driver.
fn test_default_connection() {
    connect(None).unwrap();
}
