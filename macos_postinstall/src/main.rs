use itertools::sorted;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    sync::Mutex,
};
use toml::Table;

const LOG_FILE: &str = "/tmp/postinstall_MongoDB_Atlas_SQL_ODBC.log";
const ODBC_PATH: &str = "/Library/ODBC";
const INSTALL_ROOT: &str = "/Library/MongoDB/MongoDB Atlas SQL ODBC";
const DRIVERS_SECTION: &str = "ODBC Drivers";

lazy_static! {
    pub static ref LOGGER_FILE: Mutex<File> = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&*LOG_FILE)
    {
        Err(why) => panic!("couldn't open log file {LOG_FILE:?}: {why}"),
        Ok(file) => Mutex::new(file),
    };
}

//#[cfg(not(target_os = "macos"))]
//fn main() {
//    println!("Hi, I am the install setup script for macos, I do nothing on your current os");
//}
//

fn write_to_log(data: String) {
    use std::io::Write;

    let mut logger_file = LOGGER_FILE.lock();
    while logger_file.is_err() {
        logger_file = LOGGER_FILE.lock();
    }
    let mut logger_file = logger_file.unwrap();
    println!("{}", data);
    match (*logger_file).write_all(data.as_bytes()) {
        Err(why) => panic!("couldn't write to log file {LOG_FILE:?}: {why}"),
        Ok(_) => (),
    };
    match (*logger_file).flush() {
        Err(why) => panic!("couldn't flush log file {LOG_FILE:?}: {why}"),
        Ok(_) => (),
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

fn parse_odbc_file(path: &str) -> Table {
    let mut buf = String::new();
    if std::path::Path::new(path).exists() {
        match std::fs::OpenOptions::new().read(true).open(path) {
            Err(why) => {
                err(&format!("Couldn't open {path} because {why:?}"));
                panic!()
            }
            Ok(mut file) => {
                file.read_to_string(&mut buf).unwrap();
                buf.parse::<Table>().unwrap()
            }
        }
    } else {
        Table::new()
    }
}

fn write_odbc_file(path: &str, table: Table) {
    match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
    {
        Err(why) => panic!("couldn't open log file {LOG_FILE:?}: {why}"),
        Ok(mut file) => file.write(table.to_string().as_bytes()),
    }
    .unwrap();
}

//#[cfg(target_os = "macos")]
fn main() {
    let args = env::args().collect::<Vec<_>>();
    info(&format!("{:?}", args));
    let target_volume = args[3].clone();
    let odbc_path = if target_volume != "/" {
        target_volume + "/" + ODBC_PATH
    } else {
        ODBC_PATH.to_string()
    };
    let ini_file = odbc_path.clone() + "/odbcinst.ini";
    info(&format!(
        "Drivers configuration will be added to {}",
        ini_file
    ));
    let latest = get_latest_version();
    let mdb_driver_key = format!("MongoDB Atlas SQL ODBC {latest}");
    let install_path = format!("{INSTALL_ROOT}/{latest}");
    let mdb_driver_path = format!("{install_path}/libatsql.dylib");
    info(&format!("Driver installed at: {mdb_driver_path}"));

    // create the ODBC_PATH, if it doesn't exist
    let res = fs::create_dir_all(&ini_file);
    if res.is_err() {
        err(&format!("{:?}", res));
        panic!();
    }

    let ini_table = parse_odbc_file(&odbc_path);
    info(&format!("ODBC toml = {ini_table:?}"));

    write_odbc_file(&ini_file, ini_table)
}
