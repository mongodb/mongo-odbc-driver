use crate::{col_metadata::ResultSetSchema, Error, Result};
use bson::{doc, Bson, Document};
use libloading::{Library, Symbol};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;
use std::sync::Once;

const LIBRARY_NAME: &str = "mongosqltranslate";
const MOCK_LIBRARY_NAME: &str = "mock_mongosqltranslate";

#[cfg(target_os = "windows")]
const LIBRARY_EXTENSION: &str = "dll";
#[cfg(target_os = "macos")]
const LIBRARY_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
const LIBRARY_EXTENSION: &str = "so";

// Define library installation paths for different operating systems.
// The expected library path is in the same directory as the ODBC driver.
#[cfg(target_os = "windows")]
const LIBRARY_INSTALL_PATH: &str = if cfg!(target_arch = "x86_64") {
    "C:\\Program Files\\MongoDB\\Atlas SQL ODBC Driver\\bin"
} else {
    "C:\\Program Files\\MongoDB\\ODBC\\bin"
};
#[cfg(target_os = "macos")]
const LIBRARY_INSTALL_PATH: &str = "/Library/MongoDB/MongoDB Atlas SQL ODBC Driver/";
#[cfg(target_os = "linux")]
const LIBRARY_INSTALL_PATH: &str = "/opt/mongodb/atlas-sql-odbc-driver/";

static INIT: Once = Once::new();
static mut MONGOSQLTRANSLATE_LIBRARY: Option<Library> = None;

fn get_library_name(library_type: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.{}", library_type, LIBRARY_EXTENSION)
    } else {
        format!("lib{}.{}", library_type, LIBRARY_EXTENSION)
    }
}

fn get_library_path(library_type: &str) -> PathBuf {
    let lib_name = get_library_name(library_type);
    let mut path = PathBuf::from(LIBRARY_INSTALL_PATH);
    path.push(lib_name);
    path
}

fn get_mock_library_path() -> PathBuf {
    let lib_name = get_library_name(MOCK_LIBRARY_NAME);
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("deps"); // Go to the 'deps' directory where the library should be located
    path.push(lib_name);
    path
}

// load_mongosqltranslate_library is the entry point for loading the mongosqltranslate library
// and is responsible for determining the library name and path.
// The library name and path are determined based on the operating system and architecture.
// It is stored in a static variable to ensure that it is only loaded once.
pub fn load_mongosqltranslate_library() {
    INIT.call_once(|| {
        let library_path = if cfg!(test) {
            get_mock_library_path()
        } else {
            get_library_path(LIBRARY_NAME)
        };

        match unsafe { Library::new(library_path.clone()) } {
            Ok(lib) => {
                unsafe { MONGOSQLTRANSLATE_LIBRARY = Some(lib) };
                log::info!(
                    "Loaded the mongosqltranslate library from: {}",
                    library_path.display()
                );
            }
            Err(e) => {
                log::warn!("Failed to load the mongosqltranslate library: {}", e);
            }
        }
    });
}

pub fn get_run_command_fn_ptr(
) -> std::result::Result<Symbol<'static, unsafe extern "C" fn(BsonBuffer) -> BsonBuffer>, Error> {
    let library = get_mongosqltranslate_library().ok_or(Error::UnsupportedClusterConfiguration(
        "Enterprise edition was detected, but libmongosqltranslate was not found.".to_string(),
    ))?;
    unsafe { library.get(b"runCommand") }
        .map_err(|e| Error::RunCommandSymbolNotFound(e.to_string()))
}

pub fn get_mongosqltranslate_library() -> Option<&'static Library> {
    #[allow(static_mut_refs)]
    unsafe {
        MONGOSQLTRANSLATE_LIBRARY.as_ref()
    }
}

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

impl<T: CommandName + Serialize> From<T> for Command<T> {
    fn from(options: T) -> Self {
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
        Deserialize::deserialize(deserializer)
            .map_err(Error::BsonDocumentToCommandResponseDeserialization)
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

#[repr(C)]
pub struct BsonBuffer {
    pub data: *const u8,
    pub length: usize,
    pub capacity: usize,
}

/// This function handles libmongosqltranslate runCommands. It takes in a `runCommand`,
/// handles serializing it into a BSON byte vector, calls the libmongosqltranslate runCommand,
/// deserializes the response, and returns either an error or a valid response
/// for the given `runCommand`.
pub(crate) fn libmongosqltranslate_run_command<T: CommandName + Serialize>(
    command: impl Into<Command<T>>,
) -> Result<CommandResponse> {
    let run_command_function = get_run_command_fn_ptr()?;

    let command = command.into();

    let command_bytes_vec =
        bson::to_vec(&command).map_err(Error::LibmongosqltranslateSerialization)?;

    let command_bytes_length = command_bytes_vec.len();

    let command_bytes_capacity = command_bytes_vec.capacity();

    let libmongosqltranslate_command = BsonBuffer {
        data: Box::into_raw(command_bytes_vec.into_boxed_slice()).cast(),
        length: command_bytes_length,
        capacity: command_bytes_capacity,
    };

    let decomposed_returned_doc = unsafe { run_command_function(libmongosqltranslate_command) };

    let command_response_doc: Document = unsafe {
        bson::from_slice(
            Vec::from_raw_parts(
                decomposed_returned_doc.data.cast_mut(),
                decomposed_returned_doc.length,
                decomposed_returned_doc.capacity,
            )
            .as_slice(),
        )
        .map_err(Error::LibmongosqltranslateDeserialization)?
    };

    let command_response = CommandResponse::from_document(&command_response_doc)?;

    if let CommandResponse::Error(error_response) = command_response {
        return Err(Error::LibmongosqltranslateCommandFailed(
            T::command_name(),
            error_response.error,
            error_response.error_is_internal,
        ));
    }

    Ok(command_response)
}

#[cfg(test)]
mod unit {
    use super::*;
    use bson::{doc, Document};

    #[test]
    fn library_load_and_run_command_test() {
        load_mongosqltranslate_library();
        assert!(get_mongosqltranslate_library().is_some());

        let run_command = get_run_command_fn_ptr().expect("Failed to load runCommand symbol");
        let test_doc = doc! { "test": "value" };
        let bson_bytes = bson::to_vec(&test_doc).expect("Failed to serialize BSON");

        let command_bytes_length = bson_bytes.len();

        let command_bytes_capacity = bson_bytes.capacity();

        // Call runCommand
        let command = BsonBuffer {
            data: Box::into_raw(bson_bytes.into_boxed_slice()).cast(),
            length: command_bytes_length,
            capacity: command_bytes_capacity,
        };

        let result = unsafe { run_command(command) };
        let result_vec =
            unsafe { Vec::from_raw_parts(result.data as *mut u8, result.length, result.capacity) };
        let result_doc: Document =
            bson::from_slice(&result_vec).expect("Failed to deserialize result");

        assert_eq!(result_doc, test_doc);
    }
    #[test]
    fn test_custom_serializer() {
        let translate = Translate::new("SELECT * FROM foo".to_string(), "bar".to_string(), doc! {});

        let command = Command::from(translate);

        let serialized = serde_json::to_string(&command).unwrap();

        assert_eq!(
            serialized,
            r#"{"command":"translate","options":{"sql":"SELECT * FROM foo","db":"bar","excludeNamespaces":false,"relaxSchemaChecking":true,"schemaCatalog":{}}}"#
        );
    }
}
