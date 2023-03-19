use crate::gui::config_dsn;
use cstr::input_text_to_string_w;
use mongo_odbc_core::util::dsn::DSNOpts;
use windows::Win32::{
    Foundation::HWND,
    System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN, ODBC_REMOVE_DSN},
};

/// ConfigDSN adds, modifies, or deletes data sources from the system information. It may prompt the user for connection information.
/// It can be in the driver DLL or a separate setup DLL.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
unsafe extern "C" fn ConfigDSNW(
    hwnd: HWND,
    request: u32,
    driver: *mut cstr::WideChar,
    attributes: *mut cstr::WideChar,
) -> bool {
    let mut dsn_opts =
        DSNOpts::from_attribute_string(parse_attributes(attributes)).unwrap_or_default();

    // If a data source name is passed to ConfigDSN in lpszAttributes, ConfigDSN checks that the name is valid. If the
    // data source name matches an existing data source name and hwndParent is null, ConfigDSN overwrites the existing name.
    // If it matches an existing name and hwndParent is not null, ConfigDSN prompts the user to overwrite the existing name.

    // driver can never be null. If it is, we should panic.
    dsn_opts.driver_name = unsafe { input_text_to_string_w(driver, constants::DRIVER_NAME.len()) };
    if dsn_opts.driver_name != constants::DRIVER_NAME {
        return false;
    }

    match request {
        ODBC_ADD_DSN => {
            if hwnd.0 == 0 && dsn_opts.is_valid_dsn() {
                dsn_opts.write_dsn_to_registry()
            } else {
                config_dsn(dsn_opts, request)
            }
        }
        ODBC_CONFIG_DSN => config_dsn(dsn_opts.from_private_profile_string(), request),
        ODBC_REMOVE_DSN => dsn_opts.remove_dsn(),
        _ => unreachable!(),
    }
}

fn parse_attributes(attributes: *mut cstr::WideChar) -> String {
    // 8192 is chosen here based on experimentaiton. It's long enough to hold foreseeable attributes,
    // but setting it too long causes crashes.
    let attributes = unsafe { input_text_to_string_w(attributes, 8192) }
        .split_once("\0\0")
        .unwrap()
        .0
        .to_string();
    attributes.replace(char::from(0), ";")
}
