extern crate odbc_api;

use crate::common::{connect, validate_rs};

mod common;

#[allow(dead_code)]
//#[test]
fn test_sql_tables_no_filter() {
    // Setup
    let conn = connect();

    let mut cursor;
    cursor = conn.tables(Some(""), Some(""), Some(""), Some("")).unwrap();
    validate_rs(Some(-1), None, None, &mut cursor);

    // TODO Check that clean-up does close the connection and
    // free the connection handle (connection.drop calls sqlDisconnect
    // but I did not find where it calls sqlFreeHandle)
}
