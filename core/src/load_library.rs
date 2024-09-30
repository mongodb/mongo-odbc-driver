use definitions::BsonBuffer;
use libloading::{Library, Symbol};
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

pub fn get_mock_run_command() -> Result<
    Symbol<'static, unsafe extern "C" fn(*const u8, usize) -> BsonBuffer>,
    Box<dyn std::error::Error>,
> {
    let library =
        get_mongosqltranslate_library().ok_or("mongosqltranslate library is not loaded")?;
    unsafe { library.get(b"runCommand") }.map_err(|e| e.into())
}

pub fn get_mongosqltranslate_library() -> Option<&'static Library> {
    unsafe { MONGOSQLTRANSLATE_LIBRARY.as_ref() }
}

#[cfg(test)]
mod unit {
    use super::*;
    use bson::{doc, Document};

    #[test]
    fn library_load_and_run_command_test() {
        load_mongosqltranslate_library();
        assert!(get_mongosqltranslate_library().is_some());

        let run_command = get_mock_run_command().expect("Failed to load runCommand symbol");
        let test_doc = doc! { "test": "value" };
        let bson_bytes = bson::to_vec(&test_doc).expect("Failed to serialize BSON");

        // Call runCommand
        let result = unsafe { run_command(bson_bytes.as_ptr(), bson_bytes.len()) };
        let result_vec =
            unsafe { Vec::from_raw_parts(result.data as *mut u8, result.length, result.capacity) };
        let result_doc: Document =
            bson::from_slice(&result_vec).expect("Failed to deserialize result");

        assert_eq!(result_doc, test_doc);
    }
}
