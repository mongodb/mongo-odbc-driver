#[cfg(target_os = "windows")]
pub mod windows {
    use crate::util::dsn::windows::DSNOpts;
    use file_dbg_macros::*;
    use std::io;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    const ODBC: &str = "SOFTWARE\\ODBC\\ODBC.INI";
    const ODBC_INST: &str = "SOFTWARE\\ODBC\\ODBCINST.INI";
    const ODBC_DATA_SOURCES: &str = "SOFTWARE\\ODBC\\ODBC.INI\\ODBC Data Sources";

    pub fn add_dsn(dsn_opts: &DSNOpts) -> io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (settings, _disp) = hklm
            .create_subkey(format!("{ODBC}\\{dsn}", dsn = dsn_opts.dsn))
            .unwrap();
        match settings.encode(dsn_opts) {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg_write!(format!("Error while adding dsn: {:?}", e));
                Err(io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }

    pub fn get_dsn(dsn_opts: &DSNOpts) -> Option<DSNOpts> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        hklm.open_subkey(format!("{ODBC}\\{dsn}", dsn = dsn_opts.dsn))
            .map_or(None, |key| match key.decode::<DSNOpts>() {
                Ok(dsn) => Some(dsn),
                Err(e) => {
                    dbg_write!(format!("Error while getting dsn: {:?}", e));
                    None
                }
            })
    }

    pub fn remove_dsn(dsn_opts: &DSNOpts) -> io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        match hklm.delete_subkey(format!("{ODBC}\\{dsn}", dsn = dsn_opts.dsn)) {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg_write!(format!("Error while removing dsn: {:?}", e));
                Err(e)
            }
        }
    }

    pub fn get_driver_dll_path(driver_name: &str) -> String {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm.open_subkey(format!("{ODBC_INST}\\{driver_name}"));
        match settings {
            Ok(settings) => match settings.get_value("Driver") {
                Ok(value) => value,
                Err(e) => {
                    dbg_write!(format!(
                        "Error while getting driver dll path for {}: {:?}",
                        driver_name, e
                    ));
                    "".to_string()
                }
            },
            Err(_) => "".to_string(),
        }
    }

    pub fn add_to_datasources(dsn: &str, driver: &str) -> io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (settings, _disp) = hklm.create_subkey(ODBC_DATA_SOURCES)?;
        match settings.set_value(dsn, &driver) {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg_write!(format!("Error while adding to datasources: {:?}", e));
                Err(e)
            }
        }
    }

    pub fn remove_from_datasources(dsn: &str) -> io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (settings, _disp) = hklm.create_subkey(ODBC_DATA_SOURCES)?;
        match settings.delete_value(dsn) {
            Ok(_) => Ok(()),
            Err(e) => {
                dbg_write!(format!("Error while removing from datasources: {:?}", e));
                Err(e)
            }
        }
    }

    pub fn datasource_exists(dsn: &str) -> io::Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm.open_subkey(ODBC_DATA_SOURCES);
        match settings {
            Ok(setting) => match setting.get_value::<String, _>(dsn) {
                Ok(_) => Ok(()),
                Err(e) => {
                    dbg_write!(format!(
                        "Error while getting value checking if datasource exists: {:?}",
                        e
                    ));
                    Err(io::Error::new(io::ErrorKind::Other, "Not found"))
                }
            },
            Err(e) => {
                dbg_write!(format!(
                    "Error while checking if datasource exists: {:?}",
                    e
                ));
                Err(io::Error::new(io::ErrorKind::Other, "Not found"))
            }
        }
    }
}
