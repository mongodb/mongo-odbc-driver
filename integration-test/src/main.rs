use lazy_static::lazy_static;
use mongodb::sync::Client;
use serde::{Deserialize, Serialize};
use std::{fs, io};
use thiserror::Error;

lazy_static! {
    pub static ref TEST_DATA_DIRECTORY: String = "resources/integration_test/testdata".to_string();
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

type Result<T> = std::result::Result<T, DataLoaderError>;

#[derive(Error, Debug)]
pub enum DataLoaderError {
    #[error(transparent)]
    FileSystem(#[from] io::Error),
    #[error(transparent)]
    Mongo(#[from] mongodb::error::Error),
    #[error(transparent)]
    Serde(#[from] serde_yaml::Error),
}

fn main() -> Result<()> {
    // Step 1: Read data files
    let data_files = fs::read_dir(TEST_DATA_DIRECTORY.clone())?;

    let fs = data_files
        .into_iter()
        .filter_map(|e| {
            let path = e.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "yml" {
                    return Some(path)
                }
            }
            None
        })
        .map(|p| {
            let f = fs::File::open(p)?;
            let test_data: TestData = serde_yaml::from_reader(f)?;
            Result::Ok(test_data)
        })
        .collect::<Result<Vec<TestData>>>()?;

    // let test_data: TestData = serde_yaml::from_reader(f)?;
    // println!("Read YAML file: {:?}", test_data);

    // Step 2: Delete existing data based on namespaces in data files
    // TODO

    // Step 3: Load data into mongod
    // TODO
    // let client = Client::with_uri_str("mongodb://localhost:27017")?;
    //
    // for e in test_data.dataset {
    //     let db = client.database(e.db.as_str());
    //     let collection = db.collection::<mongodb::bson::Bson>(e.collection.clone().unwrap().as_str());
    //     let res = collection.insert_many(e.docs.unwrap(), None)?;
    //     println!("insert result: {:?}", res);
    // }

    // Step 4: set schemas in ADL
    // TODO

    Ok(())
}
