use mongodb::{
    bson::{doc, Bson, Document},
    sync::{Client, Database},
    IndexModel,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, env, fs, io};
use thiserror::Error;

const TEST_DATA_DIRECTORY: &str = "resources/integration_test/testdata";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestData {
    dataset: Vec<TestDataEntry>,

    #[serde(skip)]
    file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestDataEntry {
    // db specifies the database for this test entry. Required.
    db: String,

    // collection specifies the collection for this entry. Conditional.
    // Exactly one of 'collection' or 'view' must be specified for every
    // test entry.
    collection: Option<String>,

    // view specifies the view for this test entry. Conditional.
    // Exactly one of 'collection' or 'view' must be specified for every
    // test entry. Note that ADF views are defined in ADF itself, not on
    // the underlying datasource(s) -- in this case, not on the mongod.
    // They are defined in integration_test/testdata/adl_db_config.json.
    // This data loader does not create views on the mongod. It just
    // ensures a schema is set for each view.
    view: Option<String>,

    // docs specifies the docs to insert into the collection for this
    // test entry. Conditional.
    // This is required if 'collection' is specified. The documents can
    // be specified in extended JSON format.
    docs: Option<Vec<Bson>>,

    // schema specifies the schema for this test entry. Optional.
    // If provided, this data loader sets the collection or view schema
    // using the sqlSetSchema command. If not provided, this data loader
    // sets the collection or view schema using the sqlGenerateSchema
    // command.
    schema: Option<Bson>,

    // indexes specifies the indexes for this test entry. Optional.
    // Can only be provided for collections, not views. These must be
    // specified following the Rust driver's IndexModel format:
    //   { key: <key document>, options: <options document> }
    //
    // Example:
    //   indexes:
    //     - { key: {b: 1, a: -1}}
    //
    // See the docs for more details on possible options.
    indexes: Option<Vec<IndexModel>>,
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

// This is a standalone executable that loads test data for ODBC integration tests.
// Test data should be specified in YAML files (using the .y[a]ml extension) in the
// mongo-odbc-driver/resources/integration_test/testdata directory, following the
// format described above by the TestData and TestDataEntry types. See those types
// for more details.
fn main() -> Result<()> {
    // Step 0: Read environment variables and create Mongo URIs
    let mdb_url = format!(
        "mongodb://localhost:{}",
        env::var("MDB_TEST_LOCAL_PORT").expect("MDB_TEST_LOCAL_PORT is not set")
    );
    let adf_url = format!(
        "mongodb://{}:{}@localhost",
        env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set"),
        env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set")
    );

    // Step 1: Read data files
    let test_data = read_data_files(TEST_DATA_DIRECTORY)?;

    // Step 2: Delete existing data based on namespaces in data files
    let mongod_client = Client::with_uri_str(mdb_url)?;
    drop_collections(mongod_client.clone(), test_data.clone())?;

    // Step 3: Load data into mongod. Drop everything if an error occurs.
    if let Err(e) = load_test_data(mongod_client.clone(), test_data.clone()) {
        drop_collections(mongod_client, test_data)?;
        return Err(e);
    }

    // Step 4: Set schemas in ADF
    let adf_client = Client::with_uri_str(adf_url)?;
    set_test_data_schemas(adf_client, test_data)
}

fn read_data_files(dir_path: &str) -> Result<Vec<TestData>> {
    // Read the directory and iterate over each entry
    fs::read_dir(dir_path)?
        // Only retain paths to '.y[a]ml' files
        .filter_map(|file| {
            let path = file.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    return Some(path);
                } else {
                    println!("Ignoring file without '.y[a]ml' extension: {path:?}");
                }
            }
            None
        })
        // Attempt to parse the yaml files into TestData structs
        .map(|file_path| {
            let f = fs::File::open(file_path.clone())?;
            let mut test_data: TestData = serde_yaml::from_reader(f)?;
            test_data.file = file_path.clone().into_os_string().into_string().unwrap();

            // Ensure the data files are valid. Each entry
            // must specify either a collection or a view.
            if test_data
                .clone()
                .dataset
                .into_iter()
                .filter(|entry| entry.collection.is_some() == entry.view.is_some())
                .count()
                > 0
            {
                return Err(DataLoaderError::MissingViewOrCollection(
                    file_path.into_os_string().into_string().unwrap(),
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
                .filter_map(|entry| match entry.collection {
                    Some(c) => Some((entry.db, c)),
                    None => None,
                })
                .collect::<BTreeSet<(String, String)>>()
        })
        .collect::<BTreeSet<(String, String)>>();

    for (db, c) in namespaces_to_drop {
        let database = client.database(db.as_str());
        database.collection::<Bson>(c.as_str()).drop(None)?;
        println!("Dropped {db}.{c}")
    }

    Ok(())
}

fn load_test_data(client: Client, test_data: Vec<TestData>) -> Result<()> {
    for td in test_data {
        for entry in td.dataset {
            let db = client.database(entry.db.as_str());

            if let Some(c) = entry.collection {
                let collection = db.collection::<Bson>(c.as_str());

                if let Some(docs) = entry.docs {
                    let res = collection.insert_many(docs, None)?;
                    println!(
                        "Inserted {} documents into {}.{}",
                        res.inserted_ids.len(),
                        entry.db,
                        c
                    );
                }

                if let Some(indexes) = entry.indexes {
                    let res = collection.create_indexes(indexes, None)?;
                    println!(
                        "Created indexes {:?} for {}.{}",
                        res.index_names, entry.db, c
                    );
                }
            }
        }
    }

    Ok(())
}

fn set_test_data_schemas(client: Client, test_data: Vec<TestData>) -> Result<()> {
    for td in test_data {
        for entry in td.dataset {
            let datasource = match (entry.collection, entry.view) {
                (Some(c), None) => Ok(c),
                (None, Some(v)) => Ok(v),
                _ => Err(DataLoaderError::MissingViewOrCollection(td.file.clone())),
            }?;

            let db: Database;
            let command_doc: Document;
            let command_name: &str;

            if let Some(schema) = entry.schema {
                db = client.database(entry.db.as_str());
                command_doc = doc! {"sqlSetSchema": datasource.clone(), "schema": {"jsonSchema": schema, "version": 1}};
                command_name = "sqlSetSchema";
            } else {
                db = client.database("admin");
                command_doc = doc! {"sqlGenerateSchema": 1, "setSchemas": true, "sampleNamespaces": vec![format!("{}.{}", entry.db, datasource.clone())]};
                command_name = "sqlGenerateSchema";
            }

            let res = db.run_command(command_doc, None)?;
            println!(
                "Set schema for {}.{} via {}; result: {:?}",
                entry.db, datasource, command_name, res
            );
        }
    }

    Ok(())
}
