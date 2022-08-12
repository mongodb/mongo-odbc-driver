use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

lazy_static! {
    pub static ref TEST_DATA_DIRECTORY: String = "resources/integration_test/testdata/integration_test.yml".to_string();
}

#[derive(Serialize, Deserialize, Debug)]
struct TestData {
    dataset: Vec<TestDataEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestDataEntry {
    db: String,
    collection: Option<String>,
    view: Option<String>,
    docs: Option<Vec<mongodb::bson::Bson>>,
    schema: Option<mongodb::bson::Bson>,
    indexes: Option<mongodb::bson::Bson>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open(TEST_DATA_DIRECTORY.clone())?;
    let test_data: TestData = serde_yaml::from_reader(f)?;
    println!("Read YAML file: {:?}", test_data);
    Ok(())
}
