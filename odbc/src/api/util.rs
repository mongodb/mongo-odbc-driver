use crate::definitions::ConnectionAttribute;

pub(crate) fn connection_attribute_to_string(attr: ConnectionAttribute) -> String {
    match attr {
        ConnectionAttribute::SQL_ATTR_ASYNC_ENABLE => "ASYNC_ENABLE".to_string(),
        ConnectionAttribute::SQL_ATTR_ACCESS_MODE => "ACCESS_MODE".to_string(),
        ConnectionAttribute::SQL_ATTR_AUTOCOMMIT => "AUTO_COMMIT".to_string(),
        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT => "LOGIN_TIMEOUT".to_string(),
        ConnectionAttribute::SQL_ATTR_TRACE => "TRACE".to_string(),
        ConnectionAttribute::SQL_ATTR_TRACEFILE => "TRACE_FILE".to_string(),
        ConnectionAttribute::SQL_ATTR_TRANSLATE_LIB => "TRANSLATE_LIB".to_string(),
        ConnectionAttribute::SQL_ATTR_TRANSLATE_OPTION => "TRANSLATE_OPTION".to_string(),
        ConnectionAttribute::SQL_ATTR_TXN_ISOLATION => "TXN_ISOLATION".to_string(),
        ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG => "CURRENT_CATALOG".to_string(),
        ConnectionAttribute::SQL_ATTR_ODBC_CURSORS => "ODBC_CURSORS".to_string(),
        ConnectionAttribute::SQL_ATTR_QUIET_MODE => "QUIET_MODE".to_string(),
        ConnectionAttribute::SQL_ATTR_PACKET_SIZE => "PACKET_SIZE".to_string(),
        ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT => "CONNECTION_TIMEOUT".to_string(),
        ConnectionAttribute::SQL_ATTR_DISCONNECT_BEHAVIOR => "DISCONNECT_BEHAVIOUR".to_string(),
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_FUNCTIONS_ENABLE => {
            "ASYNC_DBC_FUNCTIONS_ENABLE".to_string()
        }
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_EVENT => "ASYNC_DBC_EVENT".to_string(),
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_DTC => "ENLIST_IN_DTC".to_string(),
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_XA => "ENLIST_IN_XA".to_string(),
        ConnectionAttribute::SQL_ATTR_CONNECTION_DEAD => "CONNECTION_DEAD".to_string(),
        ConnectionAttribute::SQL_ATTR_AUTO_IPD => "AUTO_IPD".to_string(),
        ConnectionAttribute::SQL_ATTR_METADATA_ID => "METADATA_ID".to_string(),
    }
}

pub(crate) fn format_version(major: &str, minor: &str, patch: &str) -> String {
    format!(
        "{}.{}.{}",
        format_version_part(major, 2),
        format_version_part(minor, 2),
        format_version_part(patch, 4)
    )
}

fn format_version_part(part: &str, len: usize) -> String {
    if len < part.len() {
        return part.to_string();
    }
    format!("{}{}", "0".repeat(len - part.len()), part)
}

mod unit {
    #[cfg(test)]
    use super::format_version;

    macro_rules! format_version_test {
        ($func_name:ident, expected = $expected:expr, major = $major:expr, minor = $minor:expr, patch = $patch:expr) => {
            #[test]
            fn $func_name() {
                let actual = format_version($major, $minor, $patch);
                assert_eq!($expected, actual)
            }
        };
    }

    format_version_test!(
        no_padding_needed,
        expected = "10.11.1213",
        major = "10",
        minor = "11",
        patch = "1213"
    );

    format_version_test!(
        padding_needed,
        expected = "01.01.0001",
        major = "1",
        minor = "1",
        patch = "1"
    );

    format_version_test!(
        parts_larger_than_length,
        expected = "111.222.33333",
        major = "111",
        minor = "222",
        patch = "33333"
    );

    format_version_test!(
        format_cargo_version,
        expected = "00.01.0000",
        major = env!("CARGO_PKG_VERSION_MAJOR"),
        minor = env!("CARGO_PKG_VERSION_MINOR"),
        patch = env!("CARGO_PKG_VERSION_PATCH")
    );
}
