extern crate odbc_api;

use crate::common::{connect, validate_rs};
use crate::tests_gen::{load_file_paths, parse_yaml_metadata_tests, DbMetadataTest};
use lazy_static::lazy_static;
use odbc_api::handles::StatementImpl;
use odbc_api::CursorImpl;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

mod common;
mod tests_gen;

lazy_static! {
    static ref TESTS_LIST: TestsList = {
        let mut sql_tables_tests = Vec::new();
        let paths = load_file_paths(PathBuf::from("./tests/metadata_tests")).unwrap();
        for path in paths {
                    let yaml_db_metadata_tests = parse_yaml_metadata_tests(&path).unwrap();
        for test in yaml_db_metadata_tests.tests {
            match test.skip_reason {
                Some(_) => continue,
                None => {
                    let metadata_function_info = &test.meta_function;
                    let func_name = &metadata_function_info[0];
                    match func_name.as_str() {
                        "getTables" => sql_tables_tests.push(test),
                        // TODO : Implement other metadata tests
                        _ => (),
                    };
                }
            }
        }
        }
         TestsList {
            SqlTablesTests: sql_tables_tests,
            SqlColumnsTests: Vec::new(),
        }
    };
}

enum MetadataFunction {
    SqlTables,
    SqlColumns,
    SqlInfo,
}

struct TestsList {
    SqlTablesTests: Vec<DbMetadataTest>,
    SqlColumnsTests: Vec<DbMetadataTest>,
}

// TODO Add query execution test

#[test]
fn test_sql_tables() {
    // TODO - Remove comment around assert when we can run tests
    // This will make sure that at least one sqlTables tests is executed
    // assert!(!TESTS_LIST.SqlTablesTests.is_empty());
    for test in &TESTS_LIST.SqlTablesTests {
        let conn = connect();
        let mut cursor;
        let metadata_function_info = &test.meta_function;
        assert!(metadata_function_info.len() >= 5);
        cursor = conn
            .tables(
                Some(metadata_function_info[1].as_str()),
                Some(metadata_function_info[2].as_str()),
                Some(metadata_function_info[3].as_str()),
                Some(metadata_function_info[4].as_str()),
            )
            .unwrap();

        validate_rs(test.row_count, None, None, &mut cursor);

        // TODO Check that clean-up does close the connection and
        // free the connection handle (connection.drop calls sqlDisconnect
        // but I did not find where it calls sqlFreeHandle)
    }
    //}
}
