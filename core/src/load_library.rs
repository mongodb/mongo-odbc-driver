use libloading::{Library, Symbol};
use std::env;
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();
static mut LIBRARY_LOADED: bool = false;
static mut LIBRARY: Option<Library> = None;


fn get_library_path() -> PathBuf {
    let lib_name = if cfg!(target_os = "windows") {
            "libmongosqltranslate.dll"
    } else if cfg!(target_os = "macos") {
        "libmongosqltranslate.dylib"
    } else {
        "libmongosqltranslate.so"
    };

    let mut path = PathBuf::new();
    if cfg!(target_os = "windows") {
        path.push("C:\\Program Files\\MongoDB\\");
    } else if cfg!(target_os = "macos") {
        path.push("/Library/MongoDB/");
    }

    path.push(lib_name);
    path
}

fn get_library_path_test() -> PathBuf {
    let lib_name = if cfg!(target_os = "windows") {
        "libmock_mongosqltranslate.dll"
    } else if cfg!(target_os = "macos") {
        "libmock_mongosqltranslate.dylib"
    } else {
        "libmock_mongosqltranslate.so"
    };

    // testing, move the library to the target/debug directory
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop(); 
    path.push(lib_name);
    path
}

fn load_library() -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        unsafe {
            match Library::new(get_library_path_test()) {
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

        assert_eq!(result_doc, test_doc);
    }
}
