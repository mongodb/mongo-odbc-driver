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
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const LIBRARY_EXTENSION: &str = "so";

#[cfg(target_os = "windows")]
const LIBRARY_INSTALL_PATH: &str = if cfg!(target_arch = "x86_64") {
    "C:\\Program Files\\MongoDB\\Atlas SQL ODBC Driver\\bin"
} else {
    "C:\\Program Files\\MongoDB\\ODBC\\bin"
};

#[cfg(target_os = "macos")]
const LIBRARY_INSTALL_PATH: &str = "/Library/MongoDB/MongoDB Atlas SQL ODBC Driver/";

static INIT: Once = Once::new();
static mut LIBRARY_LOADED: bool = false;
static mut LIBRARY: Option<Library> = None;

fn get_library_path() -> PathBuf {
    let lib_name = if cfg!(target_os = "windows") {
        format!("{}.{}", LIBRARY_NAME, LIBRARY_EXTENSION)
    } else {
        format!("lib{}.{}", LIBRARY_NAME, LIBRARY_EXTENSION)
    };
    let mut path = PathBuf::from(LIBRARY_INSTALL_PATH);
    path.push(lib_name);
    path
}

// #[cfg(test)]
fn get_mock_library_path() -> PathBuf {
    let lib_name = if cfg!(target_os = "windows") {
        format!("{}.{}", LIBRARY_NAME, LIBRARY_EXTENSION)
    } else {
        format!("lib{}.{}", MOCK_LIBRARY_NAME, LIBRARY_EXTENSION)
    };
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove the executable name
    path.pop(); // Remove 'debug' or 'release'
    path.push("deps"); // Go to the 'deps' directory where the library should be located
    path.push(lib_name);
    path
}

pub fn load_library() -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        unsafe {
            let library_path = if cfg!(test) {
                get_mock_library_path()
            } else {
                get_library_path()
            };

            match Library::new(library_path) {
                Ok(lib) => {
                    LIBRARY = Some(lib);
                    LIBRARY_LOADED = true;
                }
                Err(e) => {
                    eprintln!("Failed to load the library: {}", e);
                }
            }
        }
    });

    if unsafe { LIBRARY_LOADED } {
        Ok(())
    } else {
        Err("Failed to load the library".into())
    }
}


fn is_library_loaded() -> bool {
    unsafe { LIBRARY_LOADED }
}

pub fn get_library() -> Option<&'static Library> {
    unsafe { LIBRARY.as_ref() }
}

pub fn get_run_command() -> Result<Symbol<'static, unsafe extern "C" fn(*const u8, usize) -> (*const u8, usize, usize)>, Box<dyn std::error::Error>> {
    load_library()?;
    unsafe {
        LIBRARY
            .as_ref()
            .ok_or("Library not loaded")?
            .get(b"runCommand")
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod lib_loading_test {
    use super::*;
    use bson::{doc, Document};

    #[test]
    fn test_run_command() {
        println!("Entering test_run_command_library");
        load_library().expect("Failed to load library");
        assert!(is_library_loaded());

        let library = get_library().expect("Failed to get library");
        // Loading the runCommand symbol
        let run_command: Symbol<unsafe extern "C" fn(*const u8, usize) -> (*const u8, usize, usize)> =
            unsafe { library.get(b"runCommand").expect("Failed to load runCommand symbol") };
        let test_doc = doc! { "test": "value" };
        let bson_bytes = bson::to_vec(&test_doc).expect("Failed to serialize BSON");

        // Call runCommand
        let (result_ptr, result_len, result_cap) = unsafe {
            run_command(bson_bytes.as_ptr(), bson_bytes.len())
        };
        let result_vec = unsafe {
            Vec::from_raw_parts(result_ptr as *mut u8, result_len, result_cap)
        };
        let result_doc: Document = bson::from_slice(&result_vec).expect("Failed to deserialize result");
        println!("Exiting test_run_command_library");

        assert_eq!(result_doc, test_doc);
    }
}
