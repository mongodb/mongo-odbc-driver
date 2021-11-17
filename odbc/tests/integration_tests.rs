use crate::tests_gen::*;
use std::path::PathBuf;

mod common;
mod tests_gen;

#[test]
fn metadata_tests() {
    let paths = load_file_paths(PathBuf::from("./tests/metadata_tests")).unwrap();
    for path in paths {
        let yaml_db_metadata_tests = parse_yaml_metadata_tests(&path).unwrap();
        for test in yaml_db_metadata_tests.tests {
            match test.skip_reason {
                Some(_) => continue,
                None => {
                    let metadata_function_info = test.meta_function;
                    let func_name = &metadata_function_info[0];
                    match func_name.as_str() {
                        "getTables" => {
                            assert!(metadata_function_info.len() >= 5);
                            common::sqltables(
                                Some(metadata_function_info[1].as_str()),
                                Some(metadata_function_info[2].as_str()),
                                Some(metadata_function_info[3].as_str()),
                                Some(metadata_function_info[4].as_str()),
                                test.row_count,
                            )
                        }
                        // TODO : Implement other metadata tests
                        _ => (), // Do nothing
                    };
                }
            }
        }
    }
}
