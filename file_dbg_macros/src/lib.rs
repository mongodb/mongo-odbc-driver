#[cfg(debug_assertions)]
use lazy_static::lazy_static;
#[cfg(debug_assertions)]
use std::{env, fs::File, sync::Mutex};

// Checks if DBG_FILE_PATH environment variable exists to use as debug file path. If not set,
// return the OS temp directory joined with default debug filename 'mongodb_odbc.log'
#[cfg(debug_assertions)]
fn get_file_path() -> String {
    if let Ok(file_path) = env::var("DBG_FILE_PATH") {
        return file_path;
    }
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.as_path();
    temp_path.join("mongodb_odbc.log").display().to_string()
}

#[cfg(debug_assertions)]
lazy_static! {
    #[derive(Debug)]
    pub static ref FILE_PATH: String =  get_file_path();

    pub static ref LOGGER_FILE: Mutex<File> =
        match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&*FILE_PATH) {
                    Err(why) => panic!("couldn't open log file {FILE_PATH:?}: {why}"),
                    Ok(file) => Mutex::new(file),
        };
}

#[macro_export]
macro_rules! dbg_write {
    () => {
        #[cfg(debug_assertions)]
        {
            use chrono::{Local, SecondsFormat};
            use file_dbg_macros::{FILE_PATH, LOGGER_FILE};
            use std::io::Write;

            let mut logger_file = LOGGER_FILE.lock();
            while logger_file.is_err() {
                logger_file = LOGGER_FILE.lock();
            }
            let mut logger_file = logger_file.unwrap();
            match (*logger_file).write_all(
                format!(
                    "{}: {}:{}\n",
                    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false),
                    file!(),
                    line!()
                )
                .as_bytes(),
            ) {
                Err(why) => panic!("couldn't write to log file {:?}: {}", FILE_PATH, why),
                Ok(_) => (),
            };
            match (*logger_file).flush() {
                Err(why) => panic!("couldn't flush log file {:?}: {}", FILE_PATH, why),
                Ok(_) => (),
            }
        }
    };
    ( $val:expr ) => {
        #[cfg(debug_assertions)]
        {
            use chrono::{Local, SecondsFormat};
            use file_dbg_macros::{FILE_PATH, LOGGER_FILE};
            use std::io::Write;

            let mut logger_file = LOGGER_FILE.lock();
            while logger_file.is_err() {
                logger_file = LOGGER_FILE.lock();
            }
            let mut logger_file = logger_file.unwrap();
            match (*logger_file).write_all(
                format!(
                    "{}: {}:{} - {:?}\n",
                    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false),
                    file!(),
                    line!(),
                    $val
                )
                .as_bytes(),
            ) {
                Err(why) => panic!("couldn't write to log file {:?}: {}", FILE_PATH, why),
                Ok(_) => (),
            };
            match (*logger_file).flush() {
                Err(why) => panic!("couldn't flush log file {:?}: {}", FILE_PATH, why),
                Ok(_) => (),
            }
        }
    };
}
