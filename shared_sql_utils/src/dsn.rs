use crate::odbcinst::*;
use cstr::{input_text_to_string_w, to_widechar_ptr};
use thiserror::Error;

const DATABASE: &str = "database";
const DSN: &str = "dsn";
const PASSWORD: &str = "password";
const PWD: &str = "pwd";
const SERVER: &str = "server";
const UID: &str = "uid";
const URI: &str = "uri";
const USER: &str = "user";
const LOGLEVEL: &str = "loglevel";
// SQL-1281
// const LOGPATH: &str = "LOGPATH";

const ODBCINI: &str = "ODBC.INI";
// The maximum length of a registry value is 16383 characters.
const MAX_VALUE_LENGTH: usize = 16383;

#[derive(Error, Debug)]
pub enum DSNError {
    #[error("Invalid DSN: {}\nDSN may not be longer than 32 characters, and may not contain any of the following characters: [ ] {{ }} ( ) , ; ? * = ! @ \\", .0)]
    Dsn(String),
    #[error(
        "The maximum length of an allowed registry value is {} characters.",
        MAX_VALUE_LENGTH
    )]
    Value,
    #[error("{}", .0)]
    Generic(String),
}

#[derive(Debug, Default)]
pub struct DSNArgs<S: Into<String> + Copy> {
    pub database: S,
    pub dsn: S,
    pub password: S,
    pub uri: S,
    pub user: S,
    pub server: S,
    pub driver_name: S,
    pub log_level: S,
}

#[derive(Debug, Default)]
pub struct DSN {
    pub database: String,
    pub dsn: String,
    pub password: String,
    pub uri: String,
    pub user: String,
    pub server: String,
    pub driver_name: String,
    pub log_level: String,
}

impl DSN {
    #[allow(clippy::too_many_arguments)]
    pub fn new<S: Into<String> + Copy>(args: DSNArgs<S>) -> Result<Self, DSNError> {
        let validation = vec![
            DSN::check_value_length(&args.database.into()),
            unsafe { SQLValidDSNW(to_widechar_ptr(&args.dsn.into()).0) },
            DSN::check_value_length(&args.password.into()),
            DSN::check_value_length(&args.uri.into()),
            DSN::check_value_length(&args.user.into()),
            DSN::check_value_length(&args.server.into()),
            DSN::check_value_length(&args.driver_name.into()),
        ];
        if validation.iter().all(|&b| b) {
            Ok(Self {
                database: args.database.into(),
                dsn: args.dsn.into(),
                password: args.password.into(),
                uri: args.uri.into(),
                user: args.user.into(),
                server: args.server.into(),
                driver_name: args.driver_name.into(),
                log_level: args.log_level.into(),
            })
        } else if !validation[1] {
            Err(DSNError::Dsn(args.dsn.into()))
        } else {
            Err(DSNError::Value)
        }
    }

    pub fn from_attribute_string(attribute_string: &str) -> Self {
        let mut dsn_opts = DSN::default();
        attribute_string.split(';').for_each(|pair| {
            let mut key_value = pair.split('=');
            let key = key_value.next().unwrap_or("");
            let value = key_value.next().unwrap_or("").replace(['{', '}'], "");
            dsn_opts.set_field(key, &value);
        });
        dsn_opts
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

    pub fn from_private_profile_string(&self) -> Result<Self, DSNError> {
        let buffer = &mut [0u16; MAX_VALUE_LENGTH];
        let mut dsn_opts = DSN::default();
        let mut error_key = "";
        let len = unsafe {
            SQLGetPrivateProfileStringW(
                to_widechar_ptr(&self.dsn).0,
                std::ptr::null(),
                to_widechar_ptr("").0,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                to_widechar_ptr(ODBCINI).0,
            )
        };
        let dsn_keys = unsafe { input_text_to_string_w(buffer.as_mut_ptr(), len as usize) };
        dsn_keys
            .split('\0')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .iter()
            .for_each(|&key| {
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

                if len > MAX_VALUE_LENGTH as i32 {
                    error_key = key;
                    return;
                }

                let value = unsafe { input_text_to_string_w(buffer.as_mut_ptr(), len as usize) };
                dsn_opts.set_field(key, &value);
            });
        // Somehow the registry value was too long. This should never happen unless Microsoft changes registry value rules.
        if !error_key.is_empty() {
            return Err(DSNError::Generic(format!("If you see this error, please report it. Attempted to read a value from registry that was too long for key: `{error_key}`.")));
        }
        dsn_opts.driver_name = self.driver_name.clone();
        dsn_opts.dsn = self.dsn.clone();
        Ok(dsn_opts)
    }

    fn set_field(&mut self, key: &str, value: &str) {
        match key.to_lowercase().as_str() {
            DATABASE => self.database = value.to_string(),
            DSN => self.dsn = value.to_string(),
            PASSWORD => self.password = value.to_string(),
            PWD => self.password = value.to_string(),
            SERVER => self.uri = value.to_string(),
            URI => self.uri = value.to_string(),
            USER => self.user = value.to_string(),
            UID => self.user = value.to_string(),
            LOGLEVEL => self.log_level = value.to_string(),
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

    pub fn to_connection_string(&self) -> String {
        self.iter()
            .map(|(key, value)| {
                if value.is_empty() {
                    return "".into();
                }
                format!("{key}={value};")
            })
            .collect::<String>()
    }
}

pub struct DSNIterator<'a> {
    inner: Vec<(&'a str, &'a str)>,
}

impl<'a> DSNIterator<'a> {
    pub fn new(dsn_opts: &'a DSN) -> Self {
        Self {
            inner: vec![
                ("Database", &dsn_opts.database),
                ("Password", &dsn_opts.password),
                ("Uri", &dsn_opts.uri),
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
        let dsn_opts = DSN::new(DSNArgs {
            database: "test",
            dsn: "test!",
            password: "test",
            uri: "test",
            user: "test",
            server: "test",
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_database_field() {
        let dsn_opts = DSN::new(DSNArgs {
            database: "t".repeat(MAX_VALUE_LENGTH + 1).as_str(),
            dsn: "test",
            password: "test",
            uri: "test",
            user: "test",
            server: "test",
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_password_field() {
        let dsn_opts = DSN::new(DSNArgs {
            database: "t",
            dsn: "test",
            password: "t".repeat(MAX_VALUE_LENGTH + 1).as_str(),
            uri: "test",
            user: "test",
            server: "test",
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_server_field() {
        let dsn_opts = DSN::new(DSNArgs {
            database: "t",
            dsn: "test",
            password: "test",
            uri: "test",
            user: "test",
            server: "t".repeat(MAX_VALUE_LENGTH + 1).as_str(),
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn invalid_value_length_in_user_field() {
        let dsn_opts = DSN::new(DSNArgs {
            database: "test",
            dsn: "test",
            password: "test",
            uri: "test",
            server: "test",
            user: "t".repeat(MAX_VALUE_LENGTH + 1).as_str(),
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_err());
    }

    #[test]
    fn valid_value_lengths_and_dsn() {
        let dsn_opts = DSN::new(DSNArgs {
            database: "test",
            dsn: "test",
            password: "test",
            uri: "test",
            server: "test",
            user: "test",
            driver_name: "test",
            log_level: "info",
        });
        assert!(dsn_opts.is_ok());
    }

    #[test]
    fn test_set_field() {
        let mut dsn_opts = DSN {
            ..Default::default()
        };
        dsn_opts.set_field("PWD", "hunter2");
        assert_eq!(dsn_opts.password, "hunter2");
        dsn_opts.set_field("pwd", "hunter3");
        assert_eq!(dsn_opts.password, "hunter3");
        dsn_opts.set_field("UID", "user1");
        assert_eq!(dsn_opts.user, "user1");
        dsn_opts.set_field("user", "user2");
        assert_eq!(dsn_opts.user, "user2");
    }
}
