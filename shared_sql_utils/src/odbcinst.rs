use crate::odbcinst::SettingError::NotFound;
use constants::DRIVER_NAME;
use cstr::{
    parse_attribute_string_a, parse_attribute_string_w, to_char_ptr, to_widechar_ptr, Char,
    WideChar,
};
use thiserror::Error;

// The maximum length of a registry value is 16383 characters, but this seems overly large.
// Considering the keys and potential values for them, 1024 seems realistically enough.
pub(crate) const MAX_VALUE_LENGTH: usize = 1024;
pub const ODBCINSTINI: &str = "ODBCINST.INI";

#[derive(Error, Debug, Clone)]
pub enum SettingError {
    #[error("Invalid DSN: {}\nDSN may not be longer than 32 characters, and may not contain any of the following characters: [ ] {{ }} ( ) , ; ? * = ! @ \\", .0)]
    Dsn(String),
    #[error(
        "The maximum length of an allowed registry value is {} characters.",
        MAX_VALUE_LENGTH
    )]
    Value,
    #[error("Section {} not found in {}", .0, .0)]
    NotFound(String, String),
    #[error("{}", .0)]
    Generic(String),
}

// The setting used to set the driver log level
pub const LOGLEVEL: &str = "loglevel";
// The setting used to set the driver path
pub const DRIVER: &str = "driver";
// The setting used to set the path for the driver setup dll
const SETUP: &str = "setup";

#[cfg_attr(target_os = "linux", link(name = "odbcinst", kind = "dylib"))]
#[cfg_attr(target_os = "macos", link(name = "iodbcinst", kind = "dylib"))]
#[cfg_attr(target_os = "windows", link(name = "odbccp32", kind = "raw-dylib"))]
extern "C" {
    pub fn SQLValidDSNW(dsn: *const WideChar) -> bool;
    pub fn SQLWriteDSNToIniW(dsn: *const WideChar, driver: *const WideChar) -> bool;
    pub fn SQLWritePrivateProfileStringW(
        section: *const WideChar,
        entry: *const WideChar,
        string: *const WideChar,
        filename: *const WideChar,
    ) -> bool;
    pub fn SQLWritePrivateProfileString(
        section: *const Char,
        entry: *const Char,
        string: *const Char,
        filename: *const Char,
    ) -> bool;
    pub fn SQLRemoveDSNFromIniW(dsn: *const WideChar) -> bool;
    pub fn SQLGetPrivateProfileStringW(
        section: *const WideChar,
        entry: *const WideChar,
        default: *const WideChar,
        buffer: *mut WideChar,
        buffer_size: i32,
        filename: *const WideChar,
    ) -> i32;
    pub fn SQLGetPrivateProfileString(
        section: *const Char,
        entry: *const Char,
        default: *const Char,
        buffer: *mut Char,
        buffer_size: i32,
        filename: *const Char,
    ) -> i32;
    pub fn SQLGetConfigMode(buffer: *mut u32) -> i32;
}

#[derive(Debug, Default)]
pub struct DriverSettings {
    pub driver: String,
    pub setup: String,
    pub log_level: String,
}

impl DriverSettings {
    // Read the odbc inst ini configuration for our driver and populate the DriverSettings struct
    // with the values read.
    pub fn from_private_profile_string() -> Result<Self, SettingError> {
        let mut driver_settings = DriverSettings::default();

        // SQLGetPrivateProfileStringW is hopelessly broken in unixodbc. As a workaround,
        // we must use SQLGetPrivateProfileString until if/when unixodbc is fixed.
        // The implication for users is that on linux, they cannot use unicode values
        // in DSN keys.
        // All odbc implementations (windows, unixodbc, iODBC) return the available
        // keys in a DSN as a null-terminated string of null-terminated strings.
        unsafe {
            let driver_keys = if cfg!(not(target_os = "linux")) {
                let wbuf = &mut [0; MAX_VALUE_LENGTH];
                wbuf.fill(0);
                let len = SQLGetPrivateProfileStringW(
                    to_widechar_ptr(DRIVER_NAME).0,
                    std::ptr::null(),
                    to_widechar_ptr("").0,
                    wbuf.as_mut_ptr(),
                    wbuf.len() as i32,
                    to_widechar_ptr(ODBCINSTINI).0,
                );

                if len > MAX_VALUE_LENGTH as i32 {
                    return Err(SettingError::Generic(format!("If you see this error, please report it. Attempted to read setting list from registry that was over {MAX_VALUE_LENGTH} characters for section: `{DRIVER_NAME}`.")));
                } else if len < 1 {
                    None
                } else {
                    Some(parse_attribute_string_w(wbuf.as_mut_ptr()))
                }
            } else {
                let abuf = &mut [0u8; MAX_VALUE_LENGTH];
                abuf.fill(0);
                let len = SQLGetPrivateProfileString(
                    to_char_ptr(DRIVER_NAME).0,
                    std::ptr::null(),
                    to_char_ptr("").0,
                    abuf.as_mut_ptr(),
                    abuf.len() as i32,
                    to_char_ptr(ODBCINSTINI).0,
                );
                if len > MAX_VALUE_LENGTH as i32 {
                    return Err(SettingError::Generic(format!("If you see this error, please report it. Attempted to read setting list from registry that was over {MAX_VALUE_LENGTH} characters for section: `{DRIVER_NAME}`.")));
                } else if len < 1 {
                    None
                } else {
                    Some(parse_attribute_string_a(abuf.as_mut_ptr()))
                }
            };

            // Get the value for each key under the DSN name section and store it in the DSN struct
            // Stops at the first error and propagate it
            match driver_keys {
                None => return Err(NotFound(DRIVER.to_string(), ODBCINSTINI.to_string())),
                Some(dsn_keys) => {
                    let keys: Vec<&str> = dsn_keys.split(';').filter(|s| !s.is_empty()).collect();
                    for key in keys {
                        if cfg!(not(target_os = "linux")) {
                            let wbuf = &mut [0; MAX_VALUE_LENGTH];
                            wbuf.fill(0);
                            let len = SQLGetPrivateProfileStringW(
                                to_widechar_ptr(DRIVER_NAME).0,
                                to_widechar_ptr(key).0,
                                to_widechar_ptr("").0,
                                wbuf.as_mut_ptr(),
                                wbuf.len() as i32,
                                to_widechar_ptr(ODBCINSTINI).0,
                            );
                            if len > MAX_VALUE_LENGTH as i32 {
                                return Err(SettingError::Generic(format!("If you see this error, please report it. Attempted to read a value from registry that was over {MAX_VALUE_LENGTH} characters for key: `{DRIVER}.{key}`.")));
                            } else if len < 1 {
                                /* Ignore keys not found since we get values from key list */
                            } else {
                                let val = parse_attribute_string_w(wbuf.as_mut_ptr());
                                driver_settings.set_field(key, &val);
                            }
                        } else {
                            let abuf = &mut [0; MAX_VALUE_LENGTH];
                            abuf.fill(0);
                            let len = SQLGetPrivateProfileString(
                                to_char_ptr(DRIVER_NAME).0,
                                to_char_ptr(key).0,
                                to_char_ptr("").0,
                                abuf.as_mut_ptr(),
                                abuf.len() as i32,
                                to_char_ptr(ODBCINSTINI).0,
                            );

                            if len > MAX_VALUE_LENGTH as i32 {
                                return Err(SettingError::Generic(format!("If you see this error, please report it. Attempted to read a value from registry that was over {MAX_VALUE_LENGTH} characters for key: `{DRIVER}.{key}`.")));
                            } else if len < 1 {
                                /* Ignore keys not found since we get values from key list */
                            } else {
                                let val = parse_attribute_string_a(abuf.as_mut_ptr());
                                driver_settings.set_field(key, &val);
                            }
                        };
                    }
                }
            }
        }
        Ok(driver_settings)
    }

    fn set_field(&mut self, key: &str, value: &str) {
        match key.to_lowercase().as_str() {
            DRIVER => self.driver = value.to_string(),
            SETUP => self.setup = value.to_string(),
            LOGLEVEL => self.log_level = value.to_string(),
            // SQL-1281
            // LOGPATH => self.logpath = value.to_string(),
            _ => { /* Ignore unexpected key */ }
        }
    }
}
