use super::{Result, TestEntry};
use odbc::{safe::AutocommitOn, Allocated, NoResult, Statement};

const GENERATED_TEST_DIR: &str = "../resources/generated_test";

/// Given a TestEntry and Statement, write the results of the test entry to
/// a file in the GENERATED_TEST_DIR. The only fields retained from the initial
/// TestEntry are description, db, and either query or meta_function.
pub fn generate_baseline_test_files(
    entry: &TestEntry,
    stmt: Statement<Allocated, NoResult, AutocommitOn>,
) -> Result<()> {
    // TODO
    Ok(())
}
