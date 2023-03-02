use crate::util::dsn::DSNOpts;
use std::io;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

const ODBC: &str = "SOFTWARE\\ODBC\\ODBC.INI";
const ODBC_INST: &str = "SOFTWARE\\ODBC\\ODBCINST.INI";
const ODBC_DATA_SOURCES: &str = "SOFTWARE\\ODBC\\ODBC.INI\\ODBC Data Sources";

pub fn set_dsn(dsn_opts: &DSNOpts) -> io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (settings, _disp) = hklm
        .create_subkey(format!("{ODBC}\\{dsn}", dsn = dsn_opts.dsn))
        .unwrap();
    match settings.encode(dsn_opts) {
        Ok(_) => Ok(()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}

pub fn get_driver_dll_path(driver: &str) -> String {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let settings = hklm.open_subkey(format!("{ODBC_INST}\\{driver}"));
    match settings {
        Ok(settings) => {
            let value: String = settings.get_value("Driver").unwrap();
            value
        }
        Err(_) => "".to_string(),
    }
}

pub fn add_datasource(dsn: &str, driver: &str) -> io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (settings, _disp) = hklm.create_subkey(ODBC_DATA_SOURCES)?;
    settings.set_value(dsn, &driver)?;
    Ok(())
}

pub fn remove_dsn(dsn: &str) -> io::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.delete_subkey(format!("{ODBC}\\{dsn}"))?;
    let (settings, _disp) = hklm.create_subkey(ODBC_DATA_SOURCES)?;
    settings.delete_value(dsn)
}
