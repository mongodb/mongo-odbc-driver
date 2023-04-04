use itertools::sorted;
use std::{env, fs};
const LOG_FILE: &str = "/tmp/postinstall_MongoDB_Atlas_SQL_ODBC.log";
const ODBC_PATH: &str = "/Library/ODBC";
const INSTALL_ROOT: &str = "/Library/MongoDB/MongoDB Atlas SQL ODBC/";
const DRIVERS_SECTION: &str = "ODBC Drivers";

//#[cfg(not(target_os = "macos"))]
//fn main() {
//    println!("Hi, I am the install setup script for macos, I do nothing on your current os");
//}
//

fn err(val: &str) {
    let data = format!("Error: {}", val);
    println!("{}", data);
    fs::write(LOG_FILE, data).expect("Unable to write to LOG_FILE");
}

fn info(val: &str) {
    let data = format!("Info: {}", val);
    println!("{}", data);
    fs::write(LOG_FILE, data).expect("Unable to write to LOG_FILE");
}

fn get_latest_version() -> String {
    let versions = fs::read_dir(INSTALL_ROOT);
    if versions.is_err() {
        err(&format!(
            "Could not read {} due to {:?}",
            INSTALL_ROOT, versions
        ));
    }
    let versions = versions.unwrap();
    let sorted_versions = sorted(
        versions
            .into_iter()
            .map(|x| {
                x.unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .split(".")
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .filter(|x| x.len() == 2)
            .map(|x| format!("{}.{}", x[0], x[1])),
    )
    .collect::<Vec<_>>();
    if sorted_versions.is_empty() {
        err(&format!("No installed versions in {}", INSTALL_ROOT));
        panic!()
    }
    sorted_versions.last().unwrap().to_string()
}

//#[cfg(target_os = "macos")]
fn main() {
    let args = env::args().collect::<Vec<_>>();
    info(&format!("{:?}", args));
    let target_volume = args[3].clone();
    let odbc_path = if !target_volume.is_empty() {
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
    let mdb_driver_key = format!("MongoDB Atlas SQL ODBC {}", latest);
    let install_path = format!("{}/{}", INSTALL_ROOT, latest);
    let mdb_driver_path = format!("{}/{}", install_path, "libatsql.dylib");

    // create the ODBC_PATH, if it doesn't exist
    let res = fs::create_dir_all(odbc_path);
    if res.is_err() {
        err(&format!("{:?}", res));
        panic!();
    }
}
