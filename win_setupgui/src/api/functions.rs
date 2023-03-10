use crate::gui::{config::config_dsn, remove::remove_dsn};
use cstr::{input_wtext_to_string, parse_string_a, parse_string_w};
use mongo_odbc_core::util::dsn::windows::DSNOpts;
use windows::Win32::{
    Foundation::HWND,
    System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN, ODBC_REMOVE_DSN},
};

const ODBC_MAX_DSN_LENGTH: usize = 32;
const INVALID_DSN_TOKENS: [&str; 14] = [
    "[", "]", "{", "}", "(", ")", ",", ";", "?", "*", "=", "!", "@", "\\",
];

/// ConfigDSN adds, modifies, or deletes data sources from the system information. It may prompt the user for connection information.
/// It can be in the driver DLL or a separate setup DLL.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn ConfigDSNW(
    hwnd: HWND,
    request: u32,
    driver: *mut cstr::WideChar,
    attributes: *mut cstr::WideChar,
) -> bool {
    // driver can never be null. If it is, we should panic.
    let mut dsn_opts = DSNOpts::new(parse_attributes(attributes)).unwrap_or_default();
    // If a data source name is passed to ConfigDSN in lpszAttributes, ConfigDSN checks that the name is valid. If the
    // data source name matches an existing data source name and hwndParent is null, ConfigDSN overwrites the existing name.
    // If it matches an existing name and hwndParent is not null, ConfigDSN prompts the user to overwrite the existing name.
    let full_dsn = dsn_opts.from_registry();
    if let Some(full_dsn) = full_dsn {
        dsn_opts = full_dsn;
    }
    dsn_opts.driver_name = unsafe { parse_string_w(driver).unwrap() };
    if !dsn_opts.dsn.is_empty() && !unsafe { SQLValidDSN(dsn_opts.dsn.clone().as_mut_ptr()) } {
        return false;
    }
    match request {
        ODBC_ADD_DSN | ODBC_CONFIG_DSN => {
            if hwnd.0 == 0 && !dsn_opts.dsn.is_empty() {
                dsn_opts.write_dsn_to_registry().is_ok()
            } else {
                config_dsn(dsn_opts, request);
                true
            }
        }
        ODBC_REMOVE_DSN => {
            remove_dsn(dsn_opts);
            true
        }
        _ => unreachable!(),
    }
}

/// SQLValidDSN checks the length and validity of the data source name before the name is added to the system information.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLValidDSN(lpszdsn: *mut odbc_sys::Char) -> bool {
    match parse_string_a(lpszdsn) {
        Some(dsn) => {
            dsn.len() <= ODBC_MAX_DSN_LENGTH
                && !INVALID_DSN_TOKENS.iter().any(|token| dsn.contains(token))
        }
        _ => false,
    }
}

/// SQLWriteDSNToIni adds a data source to the system information.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLWriteDSNToIni(
    lpszdsn: *mut cstr::Char,
    lpszdriver: *mut cstr::Char,
) -> bool {
    let dsn = parse_string_a(lpszdsn).unwrap();
    let driver = parse_string_a(lpszdriver).unwrap();
    let mut dsn_opts = DSNOpts {
        dsn,
        driver_path: driver,
        ..Default::default()
    };
    match dsn_opts.datasource_exists() {
        true => dsn_opts.remove_datasource().is_ok() && dsn_opts.add_datasource().is_ok(),
        false => dsn_opts.add_datasource().is_ok(),
    }
}

/// SQLRemoveDSNFromIni removes a data source from the system information.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLRemoveDSNFromIni(lpszdsn: *mut cstr::WideChar) -> bool {
    let dsn = parse_string_w(lpszdsn).unwrap();
    let dsn_opts = DSNOpts {
        dsn,
        ..Default::default()
    };
    dsn_opts.remove_datasource().is_ok()
}

/// SQLWritePrivateProfileString writes a value name and data to the Odbc.ini subkey of the system information.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLWritePrivateProfileString(
    _lpszsection: *mut cstr::Char,
    _lpszentry: *mut cstr::Char,
    _lpszstring: *mut cstr::Char,
    _lpszfilename: *mut cstr::Char,
) -> bool {
    unimplemented!()
}

fn parse_attributes(attributes: *mut cstr::WideChar) -> String {
    let attributes = unsafe { input_wtext_to_string(attributes, 1024) }
        .split_once("\0\0")
        .unwrap()
        .0
        .to_string();
    attributes.replace(char::from(0), ";")
}
