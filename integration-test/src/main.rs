use mongodb::{bson::Bson, sync::Client};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io};
use thiserror::Error;

const TEST_DATA_DIRECTORY: &str = "resources/integration_test/testdata";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestData {
    dataset: Vec<TestDataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestDataEntry {
    db: String,
    collection: Option<String>,
    view: Option<String>,
    docs: Option<Vec<Bson>>,
    schema: Option<Bson>,
    indexes: Option<Bson>,
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
    #[error("Each entry must specify exactly one of 'view' or 'collection', but at least one entry in {0} does not")]
    MissingViewOrCollection(String),
}

fn main() -> Result<()> {
    // Step 1: Read data files
    let testdata = read_data_files(TEST_DATA_DIRECTORY)?;

    // Step 2: Delete existing data based on namespaces in data files
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    drop_collections(client.clone(), testdata.clone())?;

    // Step 3: Load data into mongod. Drop everything if an error occurs.
    match load_test_data(client.clone(), testdata.clone()) {
        Err(e) => {
            // Drop collections if loading fails
            drop_collections(client, testdata)?;
            return Err(e);
        }
        Ok(()) => (),
    };

    // Step 4: Set schemas in ADL
    // TODO

    Ok(())
}

fn read_data_files(path: &str) -> Result<Vec<TestData>> {
    // Read the directory and iterate over each entry
    fs::read_dir(path)?
        .into_iter()
        // Only retain paths to '.yml' files
        .filter_map(|e| {
            let path = e.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "yml" {
                    return Some(path);
                }
            }
            None
        })
        // Attempt to parse the yaml files into TestData structs
        .map(|p| {
            let f = fs::File::open(p.clone())?;
            let test_data: TestData = serde_yaml::from_reader(f)?;

            // Ensure the data files are valid. Each entry
            // must specify either a collection or a view.
            if test_data
                .clone()
                .dataset
                .into_iter()
                .filter(|e| e.collection.is_some() == e.view.is_some())
                .count()
                > 0
            {
                return Err(DataLoaderError::MissingViewOrCollection(
                    p.into_os_string().into_string().unwrap(),
                ));
            }

            Result::Ok(test_data)
        })
        .collect()
}

fn drop_collections(client: Client, datasets: Vec<TestData>) -> Result<()> {
    let namespaces_to_drop = datasets
        .into_iter()
        .flat_map(|td| {
            td.dataset
                .into_iter()
                .filter_map(|e| match e.collection {
                    Some(c) => Some((e.db, c)),
                    None => None,
                })
                .collect::<BTreeSet<(String, String)>>()
        })
        .collect::<BTreeSet<(String, String)>>();

    for (db, c) in namespaces_to_drop {
        let database = client.database(db.as_str());
        database.collection::<Bson>(c.as_str()).drop(None)?;
        println!("Dropped {}.{}", db, c)
    }

    Ok(())
}

fn load_test_data(client: Client, datasets: Vec<TestData>) -> Result<()> {
    // for e in test_data.dataset {
    //     let db = client.database(e.db.as_str());
    //     let collection = db.collection::<mongodb::bson::Bson>(e.collection.clone().unwrap().as_str());
    //     let res = collection.insert_many(e.docs.unwrap(), None)?;
    //     println!("insert result: {:?}", res);
    // }
    Ok(())
}
