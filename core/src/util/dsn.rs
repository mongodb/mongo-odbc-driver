use crate::{
    odbc_uri::{
        ODBCUri,
        DATABASE,
        DSN,
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
use cstr::{input_wtext_to_string, to_widechar_ptr};
use serde::{Deserialize, Serialize};

const ODBCINI: &str = "ODBC.INI";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DSNOpts {
    pub database: String,
    pub dsn: String,
    pub password: String,
    pub server: String,
    pub user: String,
    // pub logpath: String,
    pub driver_name: String,
}

impl DSNOpts {
    pub fn new(attribs: String) -> Option<Self> {
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
        let buffer = &mut [0u16; 1024];
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
            let value = unsafe { input_wtext_to_string(buffer.as_mut_ptr(), len as usize) };
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
}

impl From<ODBCUri<'_>> for DSNOpts {
    fn from(value: ODBCUri) -> Self {
        let mut database = String::new();
        let mut dsn = String::new();
        let mut password = String::new();
        let mut server = String::new();
        let mut user = String::new();
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
            // logpath,
            ..Default::default()
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
