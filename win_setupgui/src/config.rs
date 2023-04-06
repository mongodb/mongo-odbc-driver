use crate::gui::config_dsn;
use cstr::{input_text_to_string_w, parse_attribute_string_w};
use log::error;
use shared_sql_utils::DSN;
use windows::Win32::{
    Foundation::HWND,
    System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN, ODBC_REMOVE_DSN},
};

/// ConfigDSN adds, modifies, or deletes data sources from the system information. It may prompt the user for connection information.
///
/// We do not post any errors via SQLPostIntallerErrorW from this function. If the user supplies an invalid DSN via the UI,
/// we show a message box and allow them to recover. This is better than our competitors who dump all input and return the
/// user to the beginning of the flow with a simple message that says "Invalid DSN".
///
/// Other errors are not queried, and we will return false if any of the other Installer functions return false.
///
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
    std::panic::catch_unwind(|| {
        let mut dsn_opts = DSN::from_attribute_string(&parse_attribute_string_w(attributes));

        // If a data source name is passed to ConfigDSN in lpszAttributes, ConfigDSN checks that the name is valid. If the
        // data source name matches an existing data source name and hwndParent is null, ConfigDSN overwrites the existing name.
        // If it matches an existing name and hwndParent is not null, ConfigDSN prompts the user to overwrite the existing name.

        dsn_opts.driver_name =
            unsafe { input_text_to_string_w(driver, constants::DRIVER_NAME.len()) };

        match request {
            ODBC_ADD_DSN => {
                if hwnd.0 == 0 && dsn_opts.is_valid_dsn() {
                    dsn_opts.write_dsn_to_registry()
                } else {
                    config_dsn(dsn_opts, request)
                }
            }
            ODBC_CONFIG_DSN => match dsn_opts.from_private_profile_string() {
                Ok(dsn) => config_dsn(dsn, request),
                Err(e) => {
                    // we've somehow attempted to read a value from the DSN
                    // that is longer than the registry allows (at the time of writing!)
                    error!("Error reading DSN: {e}");
                    false
                }
            },
            ODBC_REMOVE_DSN => dsn_opts.remove_dsn(),
            _ => unreachable!(),
        }
    })
    .unwrap_or(false)
}
