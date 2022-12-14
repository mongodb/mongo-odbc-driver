use integration_test::test_runner::{run_integration_tests, Result};
use odbc::{safe::AutocommitOn, Allocated, NoResult, Statement};

/// This is a standalone executable that generates baseline integration test
/// files based on the description, db, and either the query or meta_function
/// fields for all test cases in the resources/integration_test/testes
/// directory. It does this by executing the integration tests and writing
/// the results to a file instead of validating the results match expectation.
///
/// The actual code for generating the test files can be found in the
/// test_runner module.
fn main() -> Result<()> {
    run_integration_tests(true)
}
