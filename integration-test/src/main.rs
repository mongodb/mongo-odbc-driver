use lazy_static::lazy_static;
use mongodb::sync::Client;
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

    let client = Client::with_uri_str("mongodb://localhost:27017")?;

    for e in test_data.dataset {
        let db = client.database(e.db.as_str());
        let collection = db.collection::<mongodb::bson::Bson>(e.collection.clone().unwrap().as_str());
        let res = collection.insert_many(e.docs.unwrap(), None)?;
        println!("insert result: {:?}", res);
    }

    Ok(())
}
