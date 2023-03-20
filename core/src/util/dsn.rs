use crate::{
    odbc_uri::{
        ODBCUri,
        DATABASE,
        DSN,
        // SQL-1281
        // LOGPATH,
        PASSWORD,
        PWD,
        SERVER,
        UID,
        URI,
        USER,
    },
    util::odbcinst::*,
};
use cstr::{input_text_to_string_w, to_widechar_ptr};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const ODBCINI: &str = "ODBC.INI";
const BASE_SYSTEM_KEY: &str = "HKEY_LOCAL_MACHINE\\SOFTWARE\\ODBC\\ODBC.INI\\";
const BASE_USER_KEY: &str = "HKEY_CURRENT_USER\\SOFTWARE\\ODBC\\ODBC.INI\\";
// The maximum length of a registry value is 16383 characters.
const MAX_VALUE_LENGTH: usize = 16383;

#[derive(Error, Debug)]
pub enum DSNError {
    #[error("Invalid DSN: {}\nDSN may not be longer than 32 characters, and may not contain any of the following characters: [ ] {{ }} ( ) , ; ? * = ! @ \\", .0)]
    DSN(String),
    #[error(
        "The maximum length of an allowed registry value is {} characters.",
        MAX_VALUE_LENGTH
    )]
    Value,
    #[error("{}", .0)]
    Generic(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DSNOpts {
    pub database: String,
    pub dsn: String,
    pub password: String,
    pub server: String,
    pub user: String,
    // SQL-1281
    // pub logpath: String,
    pub driver_name: String,
}

impl DSNOpts {
    pub fn new(
        database: String,
        dsn: String,
        password: String,
        server: String,
        user: String,
        driver_name: String,
    ) -> Result<Self, DSNError> {
        match (
            DSNOpts::check_value_length(&database),
            unsafe { SQLValidDSNW(to_widechar_ptr(&dsn).0) },
            DSNOpts::check_value_length(&password),
            DSNOpts::check_value_length(&server),
            DSNOpts::check_value_length(&user),
            DSNOpts::check_value_length(&driver_name),
        ) {
            (true, true, true, true, true, true) => Ok(Self {
                database,
                dsn,
                password,
                server,
                user,
                driver_name,
            }),
            (_, false, _, _, _, _) => Err(DSNError::DSN(dsn)),
            _ => Err(DSNError::Value),
        }
    }
    pub fn from_attribute_string(attribs: String) -> Option<Self> {
        match ODBCUri::new(&attribs.replace(char::from(0), ";")) {
            Ok(uri) => Some(Self::from(uri)),
            Err(_) => None,
        }
    }

    pub fn is_valid_dsn(&self) -> bool {
        unsafe { SQLValidDSNW(to_widechar_ptr(&self.dsn).0) }
    }

    pub fn write_dsn_to_registry(&self) -> bool {
        (unsafe {
            SQLWriteDSNToIniW(
                to_widechar_ptr(&self.dsn).0,
                to_widechar_ptr(&self.driver_name).0,
            )
        }) && self.write_private_profile_string()
    }

    fn write_private_profile_string(&self) -> bool {
        self.iter().all(|(key, value)| unsafe {
            SQLWritePrivateProfileStringW(
                to_widechar_ptr(&self.dsn).0,
                to_widechar_ptr(key).0,
                to_widechar_ptr(value).0,
                to_widechar_ptr(ODBCINI).0,
            )
        })
    }

    pub fn from_private_profile_string(&self) -> Self {
        let buffer = &mut [0u16; MAX_VALUE_LENGTH];
        let mut dsn_opts = DSNOpts::default();
        self.iter().for_each(|(key, _)| {
            let len = unsafe {
                SQLGetPrivateProfileStringW(
                    to_widechar_ptr(&self.dsn).0,
                    to_widechar_ptr(key).0,
                    to_widechar_ptr("").0,
                    buffer.as_mut_ptr(),
                    buffer.len() as i32,
                    to_widechar_ptr(ODBCINI).0,
                )
            };
            let value = unsafe { input_text_to_string_w(buffer.as_mut_ptr(), len as usize) };
            dsn_opts.set_field(key, &value);
        });
        dsn_opts.driver_name = self.driver_name.clone();
        dsn_opts.dsn = self.dsn.clone();
        dsn_opts
    }

    fn set_field(&mut self, key: &str, value: &str) {
        match key.to_lowercase().as_str() {
            DATABASE => self.database = value.to_string(),
            DSN => self.dsn = value.to_string(),
            PASSWORD => self.password = value.to_string(),
            PWD => self.password = value.to_string(),
            SERVER => self.server = value.to_string(),
            URI => self.server = value.to_string(),
            USER => self.user = value.to_string(),
            UID => self.user = value.to_string(),
            // SQL-1281
            // LOGPATH => self.logpath = value.to_string(),
            _ => {}
        }
    }

    pub fn remove_dsn(&self) -> bool {
        unsafe { SQLRemoveDSNFromIniW(to_widechar_ptr(&self.dsn).0) }
    }

    fn iter(&self) -> DSNIterator<'_> {
        DSNIterator::new(self)
    }

    fn check_value_length(value: &str) -> bool {
        value.len() < MAX_VALUE_LENGTH
    }
}

impl From<ODBCUri<'_>> for DSNOpts {
    fn from(value: ODBCUri) -> Self {
        let mut database = String::new();
        let mut dsn = String::new();
        let mut password = String::new();
        let mut server = String::new();
        let mut user = String::new();
        // SQL-1281
        // let mut logpath = String::new();
        for (key, value) in value.iter() {
            match key.to_lowercase().as_str() {
                DATABASE => database = value.to_string(),
                DSN => dsn = value.to_string(),
                PASSWORD => password = value.to_string(),
                PWD => password = value.to_string(),
                SERVER => server = value.to_string(),
                URI => server = value.to_string(),
                USER => user = value.to_string(),
                UID => user = value.to_string(),
                // SQL-1281
                // LOGPATH => logpath = value.to_string(),
                _ => {}
            }
        }
        DSNOpts {
            database,
            dsn,
            password,
            server,
            user,
            // SQL-1281
            // logpath,
            driver_name: constants::DRIVER_NAME.to_string(),
        }
    }
}

pub struct DSNIterator<'a> {
    inner: Vec<(&'a str, &'a str)>,
}

impl<'a> DSNIterator<'a> {
    pub fn new(dsn_opts: &'a DSNOpts) -> Self {
        Self {
            inner: vec![
                ("Database", &dsn_opts.database),
                ("Password", &dsn_opts.password),
                ("Server", &dsn_opts.server),
                ("User", &dsn_opts.user),
                // SQL-1281
                // ("Logpath", &dsn_opts.logpath),
            ],
        }
    }
}

impl<'a> Iterator for DSNIterator<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invalid_dsn_name() {
        let dsn_opts = DSNOpts::new(
            "test".into(),
            "test!".into(),
            "test".into(),
            "test".into(),
            "test".into(),
            "test".into(),
        );
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_database_field() {
        let dsn_opts = DSNOpts::new(
            "t".repeat(MAX_VALUE_LENGTH + 1).into(),
            "test".into(),
            "test".into(),
            "test".into(),
            "test".into(),
            "test".into(),
        );
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_password_field() {
        let dsn_opts = DSNOpts::new(
            "test".into(),
            "test".into(),
            "t".repeat(MAX_VALUE_LENGTH + 1).into(),
            "test".into(),
            "test".into(),
            "test".into(),
        );
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_server_field() {
        let dsn_opts = DSNOpts::new(
            "test".into(),
            "test".into(),
            "test".into(),
            "t".repeat(MAX_VALUE_LENGTH + 1).into(),
            "test".into(),
            "test".into(),
        );
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_user_field() {
        let dsn_opts = DSNOpts::new(
            "test".into(),
            "test".into(),
            "test".into(),
            "test".into(),
            "t".repeat(MAX_VALUE_LENGTH).into(),
            "test".into(),
        );
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn valid_value_lengths_and_dsn() {
        let dsn_opts = DSNOpts::new(
            "t".repeat(MAX_VALUE_LENGTH - 1).into(),
            "test".into(),
            "t".repeat(MAX_VALUE_LENGTH - 1).into(),
            "t".repeat(MAX_VALUE_LENGTH - 1).into(),
            "t".repeat(MAX_VALUE_LENGTH - 1).into(),
            "test".into(),
        );
        assert!(dsn_opts.is_ok());
    }
}
