use ini::Ini;
use itertools::sorted;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io::Write,
    sync::Mutex,
};

const LOG_FILE: &str = "/tmp/postinstall_MongoDB_Atlas_SQL_ODBC.log";
const ODBC_PATH: &str = "/Library/ODBC";
const INSTALL_ROOT: &str = "/Library/MongoDB/MongoDB Atlas SQL ODBC Driver";
const DRIVERS_SECTION: &str = "ODBC Drivers";

lazy_static! {
    pub static ref LOGGER_FILE: Mutex<File> = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
    {
        Err(why) => panic!("couldn't open log file {LOG_FILE:?}: {why}"),
        Ok(file) => Mutex::new(file),
    };
}

fn write_to_log(data: String) {
    let mut logger_file = LOGGER_FILE.lock();
    while logger_file.is_err() {
        logger_file = LOGGER_FILE.lock();
    }
    let mut logger_file = logger_file.unwrap();
    if let Err(why) = (*logger_file).write_all(data.as_bytes()) {
        panic!("couldn't write to log file {LOG_FILE:?}: {why}");
    };
    if let Err(why) = (*logger_file).flush() {
        panic!("couldn't flush log file {LOG_FILE:?}: {why}");
    }
}

fn err(val: &str) {
    let data = format!("Error: {val}\n");
    write_to_log(data);
}

fn info(val: &str) {
    let data = format!("Info: {val}\n");
    write_to_log(data);
}

fn get_latest_version() -> String {
    let versions = fs::read_dir(INSTALL_ROOT);
    if versions.is_err() {
        err(&format!(
            "Could not read {INSTALL_ROOT} due to {versions:?}"
        ));
    }
    let versions = versions.unwrap();
    let sorted_versions = sorted(versions.into_iter().map(|x| {
        let x = x.unwrap();
        (
            x.metadata().unwrap().modified().unwrap(),
            x.file_name().into_string().unwrap(),
        )
    }))
    .collect::<Vec<_>>();
    if sorted_versions.is_empty() {
        err(&format!("No installed versions in {INSTALL_ROOT}"));
        panic!()
    }
    sorted_versions.last().unwrap().1.to_string()
}

fn parse_odbc_file(path: &str) -> Ini {
    if std::path::Path::new(path).exists() {
        Ini::load_from_file(path).unwrap()
    } else {
        Ini::new()
    }
}

fn write_odbc_file(path: &str, ini: Ini) {
    ini.write_to_file(path).unwrap()
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let odbc_path = if args.len() > 3 && args[3] != "/" {
        args[3].clone() + "/" + ODBC_PATH
    } else {
        ODBC_PATH.to_string()
    };
    let ini_file = odbc_path.clone() + "/odbcinst.ini";
    info(&format!(
        "Drivers configuration will be added to {ini_file}"
    ));
    let latest = get_latest_version();
    let mdb_driver_key = "MongoDB Atlas SQL ODBC Driver".to_string();
    let install_path = format!("{INSTALL_ROOT}/{latest}");
    let mdb_driver_path = format!("{install_path}/libatsql.dylib");
    info(&format!("Driver installed at: {mdb_driver_path}"));

    // create the ODBC_PATH, if it doesn't exist
    let res = fs::create_dir_all(&odbc_path);
    if res.is_err() {
        err(&format!("Failed to create ODBC_PATH: {res:?}"));
        panic!();
    }

    let mut ini = parse_odbc_file(&ini_file);
    let drivers_section = ini.section_mut(Some(DRIVERS_SECTION));
    if let Some(drivers_section) = drivers_section {
        drivers_section.insert(mdb_driver_key.clone(), "Installed");
    } else {
        ini.with_section(Some(DRIVERS_SECTION))
            .set(mdb_driver_key.clone(), "Installed");
    }

    ini.with_section(Some(mdb_driver_key))
        .set(
            "Description",
            format!("MongoDB Atlas SQL ODBC Driver {latest}"),
        )
        .set("Driver", mdb_driver_path)
        .set("DriverUnicodeType", "utf16");

    info(&format!("Writing ini_file: {ini_file}"));
    write_odbc_file(&ini_file, ini)
}
