#[cfg(target_os = "windows")]
pub mod windows {
    use crate::{
        odbc_uri::{
            ODBCUri, DATABASE, DRIVER, DSN, LOGPATH, PASSWORD, PWD, SERVER, UID, URI, USER,
        },
        util::registry::windows::{add_datasource, get_driver_dll_path, remove_dsn, set_dsn},
    };
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct DSNOpts {
        #[serde(rename = "Database")]
        pub database: String,
        #[serde(rename = "Driver")]
        pub driver: String,
        #[serde(rename = "DSN")]
        pub dsn: String,
        #[serde(rename = "Password")]
        pub password: String,
        #[serde(rename = "Server")]
        pub server: String,
        #[serde(rename = "User")]
        pub user: String,
        #[serde(rename = "Logpath")]
        pub logpath: String,
        #[serde(skip_serializing)]
        pub driver_name: String,
    }

    impl DSNOpts {
        pub fn new<T: Into<String>>(attribs: T) -> Option<Self> {
            match ODBCUri::new(&attribs.into().replace(char::from(0), ";")) {
                Ok(uri) => Some(Self::from(uri)),
                Err(_) => None,
            }
        }

        pub fn write_to_registry(&mut self) -> std::io::Result<()> {
            if self.driver.is_empty() {
                self.driver = get_driver_dll_path(&self.driver_name);
            }
            set_dsn(self)?;
            add_datasource(&self.dsn, &self.driver_name)
        }

        pub fn remove_from_registry(&self) -> std::io::Result<()> {
            remove_dsn(&self.dsn)
        }
    }

    impl From<ODBCUri<'_>> for DSNOpts {
        fn from(value: ODBCUri) -> Self {
            let mut database = String::new();
            let mut driver = String::new();
            let mut dsn = String::new();
            let mut password = String::new();
            let mut server = String::new();
            let mut user = String::new();
            let mut logpath = String::new();
            for (key, value) in value.into_iter() {
                match key.to_lowercase().as_str() {
                    DATABASE => database = value.to_string(),
                    DRIVER => driver = value.to_string(),
                    DSN => dsn = value.to_string(),
                    PASSWORD => password = value.to_string(),
                    PWD => password = value.to_string(),
                    SERVER => server = value.to_string(),
                    URI => server = value.to_string(),
                    USER => user = value.to_string(),
                    UID => user = value.to_string(),
                    LOGPATH => logpath = value.to_string(),
                    _ => {}
                }
            }
            DSNOpts {
                database,
                driver,
                dsn,
                password,
                server,
                user,
                logpath,
                ..Default::default()
            }
        }
    }
}
