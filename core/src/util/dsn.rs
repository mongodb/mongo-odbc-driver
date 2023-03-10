#[cfg(target_os = "windows")]
pub mod windows {
    use crate::{
        odbc_uri::{
            ODBCUri, DATABASE, DRIVER, DSN, LOGPATH, PASSWORD, PWD, SERVER, UID, URI, USER,
        },
        util::registry::windows::{
            add_dsn, add_to_datasources, datasource_exists, get_driver_dll_path, get_dsn,
            remove_dsn, remove_from_datasources,
        },
    };
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct DSNOpts {
        #[serde(rename = "Database", default = "String::new")]
        pub database: String,
        #[serde(rename = "Driver", default = "String::new")]
        pub driver_path: String,
        #[serde(rename = "DSN", default = "String::new")]
        pub dsn: String,
        #[serde(rename = "Password", default = "String::new")]
        pub password: String,
        #[serde(rename = "Server", default = "String::new")]
        pub server: String,
        #[serde(rename = "User", default = "String::new")]
        pub user: String,
        #[serde(rename = "Logpath", default = "String::new")]
        pub logpath: String,
        #[serde(skip_serializing, skip_deserializing)]
        pub driver_name: String,
    }

    impl DSNOpts {
        pub fn new(attribs: String) -> Option<Self> {
            match ODBCUri::new(&attribs.replace(char::from(0), ";")) {
                Ok(uri) => Some(Self::from(uri)),
                Err(_) => None,
            }
        }

        pub fn write_dsn_to_registry(&mut self) -> std::io::Result<()> {
            if self.driver_path.is_empty() {
                self.driver_path = get_driver_dll_path(&self.driver_name);
            }
            add_dsn(self)
        }

        pub fn delete_dsn_from_registry(&self) -> std::io::Result<()> {
            remove_dsn(self)
        }

        pub fn from_registry(&mut self) -> Option<Self> {
            if self.dsn.is_empty() {
                return None;
            }
            get_dsn(self)
        }

        pub fn remove_datasource(&self) -> std::io::Result<()> {
            remove_from_datasources(&self.dsn)
        }

        pub fn add_datasource(&mut self) -> std::io::Result<()> {
            add_to_datasources(&self.dsn, &self.driver_name)
        }

        pub fn datasource_exists(&self) -> bool {
            datasource_exists(&self.dsn).is_ok()
        }
    }

    impl From<ODBCUri<'_>> for DSNOpts {
        fn from(value: ODBCUri) -> Self {
            let mut database = String::new();
            let mut driver_path = String::new();
            let mut dsn = String::new();
            let mut password = String::new();
            let mut server = String::new();
            let mut user = String::new();
            let mut logpath = String::new();
            for (key, value) in value.iter() {
                match key.to_lowercase().as_str() {
                    DATABASE => database = value.to_string(),
                    DRIVER => driver_path = value.to_string(),
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
                driver_path,
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
