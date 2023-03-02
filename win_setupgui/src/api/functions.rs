use crate::gui::{config::config_dsn, remove::remove_dsn};
use mongo_odbc_core::util::dsn::DSNOpts;
use mongoodbc::{input_text_to_string, input_wtext_to_string};
use windows::Win32::{
    Foundation::HWND,
    System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN, ODBC_REMOVE_DSN},
};

const ODBC_MAX_DSN_LENGTH: usize = 32;
const INVALID_DSN_TOKENS: [&str; 14] = [
    "[", "]", "{", "}", "(", ")", ",", ";", "?", "*", "=", "!", "@", "\\",
];

#[no_mangle]
pub extern "C" fn ConfigDSNW(
    _: HWND,
    request: u32,
    driver: *mut widechar::WideChar,
    attributes: *mut widechar::WideChar,
) -> bool {
    let uri_opts = DSNOpts::new(parse_attributes(attributes)).map_or(
        DSNOpts {
            // we should never encounter a situation where the driver isn't set. If it is missing for some reason, we should panic.
            driver: parse_driver(driver).unwrap(),
            ..Default::default()
        },
        |mut opts| {
            // we should never encounter a situation where the driver isn't set. If it is missing for some reason, we should panic.
            opts.driver = parse_driver(driver).unwrap();
            opts
        },
    );
    match request {
        ODBC_ADD_DSN | ODBC_CONFIG_DSN => {
            config_dsn(uri_opts, request);
            true
        }
        ODBC_REMOVE_DSN => {
            remove_dsn(uri_opts);
            true
        }
        _ => unreachable!(),
    }
}

#[no_mangle]
pub extern "C" fn SQLValidDSN(lpszdsn: *mut odbc_sys::Char) -> bool {
    match parse_string_a(lpszdsn) {
        Some(dsn) => {
            dsn.len() <= ODBC_MAX_DSN_LENGTH
                && !INVALID_DSN_TOKENS.iter().any(|token| dsn.contains(token))
        }
        _ => false,
    }
}

fn parse_attributes(attributes: *mut widechar::WideChar) -> String {
    let attributes = unsafe { input_wtext_to_string(attributes, 1024) }
        .split_once("\0\0")
        .unwrap()
        .0
        .to_string();
    attributes.replace(char::from(0), ";")
}

fn parse_string_a(str: *mut odbc_sys::Char) -> Option<String> {
    let string = unsafe { input_text_to_string(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

fn parse_string_w(str: *mut widechar::WideChar) -> Option<String> {
    let string = unsafe { input_wtext_to_string(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

fn parse_driver(driver: *mut widechar::WideChar) -> Option<String> {
    parse_string_w(driver)
}

/*
BOOL ConfigDriver(
      HWND    hwndParent,
      WORD    fRequest,
      LPCSTR  lpszDriver,
      LPCSTR  lpszArgs,
      LPSTR   lpszMsg,
      WORD    cbMsgMax,
      WORD *  pcbMsgOut);
      */
