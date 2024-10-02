use crate::{col_metadata::ResultSetSchema, Error, Result};
use bson::{doc, Bson, Document};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub trait CommandName {
    fn command_name() -> &'static str;
}

#[derive(Debug)]
pub struct Command<T> {
    pub options: T,
}
impl<T> serde::ser::Serialize for Command<T>
where
    T: CommandName + serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("command", T::command_name())?;
        map.serialize_entry("options", &self.options)?;
        map.end()
    }
}

impl<T> Command<T> {
    pub fn new(options: T) -> Self {
        Self { options }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNamespaces {
    pub sql: String,
    pub db: String,
}

impl CommandName for GetNamespaces {
    fn command_name() -> &'static str {
        "getNamespaces"
    }
}

impl GetNamespaces {
    pub fn new(sql: String, db: String) -> Self {
        Self { sql, db }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translate {
    pub sql: String,
    pub db: String,
    #[serde(rename = "excludeNamespaces")]
    pub exclude_namespaces: bool,
    #[serde(rename = "relaxSchemaChecking")]
    pub relax_schema_checking: bool,
    #[serde(rename = "schemaCatalog")]
    pub schema_catalog: Document,
}

impl CommandName for Translate {
    fn command_name() -> &'static str {
        "translate"
    }
}

impl Translate {
    pub fn new(sql: String, db: String, schema_catalog: Document) -> Self {
        Self {
            sql,
            db,
            exclude_namespaces: false,
            relax_schema_checking: true,
            schema_catalog,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMongosqlTranslateVersion {}

impl CommandName for GetMongosqlTranslateVersion {
    fn command_name() -> &'static str {
        "getMongosqlTranslateVersion"
    }
}

impl GetMongosqlTranslateVersion {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GetMongosqlTranslateVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckDriverVersion {
    #[serde(rename = "odbcDriver")]
    pub odbc_driver: bool,
    #[serde(rename = "driverVersion")]
    pub driver_version: String,
}

impl CommandName for CheckDriverVersion {
    fn command_name() -> &'static str {
        "checkDriverVersion"
    }
}

impl CheckDriverVersion {
    pub fn new(driver_version: String) -> Self {
        Self {
            odbc_driver: true,
            driver_version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandResponse {
    Translate(TranslateCommandResponse),
    GetNamespaces(GetNamespacesCommandResponse),
    GetMongosqlTranslateVersion(GetMongosqlTranslateVersionCommandResponse),
    CheckDriverVersion(CheckDriverVersionCommandResponse),
    Error(ErrorResponse),
}

impl CommandResponse {
    pub fn from_document(doc: &Document) -> Result<Self> {
        let as_bson = Bson::Document(doc.clone());
        let deserializer = bson::Deserializer::new(as_bson);
        let deserializer = serde_stacker::Deserializer::new(deserializer);
        Deserialize::deserialize(deserializer).map_err(Error::LibmongosqltranslateDeserialization)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_is_internal: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateCommandResponse {
    pub target_db: String,
    pub target_collection: Option<String>,
    pub pipeline: bson::Bson,
    #[serde(flatten)]
    pub result_set_schema: ResultSetSchema,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Namespace {
    pub database: String,
    pub collection: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNamespacesCommandResponse {
    pub namespaces: BTreeSet<Namespace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMongosqlTranslateVersionCommandResponse {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckDriverVersionCommandResponse {
    pub compatible: bool,
}

#[test]
fn test_custom_serializer() {
    let command = Command::new(Translate::new(
        "SELECT * FROM foo".to_string(),
        "bar".to_string(),
        doc! {},
    ));

    let serialized = serde_json::to_string(&command).unwrap();

    assert_eq!(
        serialized,
        r#"{"command":"translate","options":{"sql":"SELECT * FROM foo","db":"bar","excludeNamespaces":false,"relaxSchemaChecking":true,"schemaCatalog":{}}}"#
    );
}
