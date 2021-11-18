use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("failed to read directory: {0}")]
    InvalidDirectory(String),
    #[error("failed to load file paths: {0}")]
    InvalidFilePath(String),
    #[error("failed to read file: {0}")]
    InvalidFile(String),
    #[error("unable to read file to string: {0}")]
    CannotReadFileToString(String),
    #[error("unable to deserialize YAML file: {0}")]
    CannotDeserializeYaml(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlQueryTests {
    pub tests: Vec<QueryTest>,
}

// TODO
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryTest {
    pub description: String,
    pub row_count: String,
    pub rowcount_gte: String,
    pub expected_results: Vec<Vec<String>>,
    pub expected_sql_types: Vec<String>,
    pub expected_bson_type: Vec<String>,
    pub expected_catalog_name: Vec<String>,
    pub expected_column_class_name: Vec<String>,
    pub expected_column_display_size: Vec<String>,
    pub expected_column_label: Vec<String>,
    pub expected_column_type: Vec<String>,
    pub expected_precision: Vec<String>,
    pub expected_scale: Vec<String>,
    pub expected_schema_name: Vec<String>,
    pub expected_is_auto_increment: Vec<String>,
    pub expected_is_case_sensitive: Vec<String>,
    pub expected_is_currency: Vec<String>,
    pub expected_is_definitely_writable: Vec<String>,
    pub expected_is_nullable: Vec<String>,
    pub expected_is_read_only: Vec<String>,
    pub expected_is_searchable: Vec<String>,
    pub expected_is_signed: Vec<String>,
    pub expected_is_writable: Vec<String>,
    pub expected_names: Vec<String>,
    pub ordered: bool,
    pub sql: String,
    pub meta_function: String,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlDbMetadataTests {
    pub tests: Vec<DbMetadataTest>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DbMetadataTest {
    pub description: String,
    pub row_count: Option<i64>,
    pub rowcount_gte: Option<i64>,
    pub expected_results: Vec<Vec<String>>,
    pub meta_function: Vec<String>,
    pub skip_reason: Option<String>,
}

pub fn load_file_paths(dir: PathBuf) -> Result<Vec<String>, Error> {
    let mut paths: Vec<String> = vec![];
    let entries = fs::read_dir(dir).map_err(|e| Error::InvalidDirectory(format!("{:?}", e)))?;
    for entry in entries {
        match entry {
            Ok(de) => {
                let path = de.path();
                if path.extension().unwrap() == "yml" {
                    paths.push(path.to_str().unwrap().to_string());
                }
            }
            Err(e) => return Err(Error::InvalidFilePath(format!("{:?}", e))),
        };
    }
    Ok(paths)
}

pub fn parse_yaml_metadata_tests(path: &str) -> Result<YamlDbMetadataTests, Error> {
    let mut f = fs::File::open(path).map_err(|e| Error::InvalidFile(format!("{:?}", e)))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|e| Error::CannotReadFileToString(format!("{:?}", e)))?;
    let yaml: YamlDbMetadataTests = serde_yaml::from_str(&contents)
        .map_err(|e| Error::CannotDeserializeYaml(format!("{:?}", e)))?;
    Ok(yaml)
}
