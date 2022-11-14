use odbc_sys::ConnectionAttribute;

pub(crate) fn connection_attribute_to_string(attr: ConnectionAttribute) -> String {
    match attr {
        ConnectionAttribute::AsyncEnable => "ASYNC_ENABLE".to_string(),
        ConnectionAttribute::AccessMode => "ACCESS_MODE".to_string(),
        ConnectionAttribute::AutoCommit => "AUTO_COMMIT".to_string(),
        ConnectionAttribute::LoginTimeout => "LOGIN_TIMEOUT".to_string(),
        ConnectionAttribute::Trace => "TRACE".to_string(),
        ConnectionAttribute::TraceFile => "TRACE_FILE".to_string(),
        ConnectionAttribute::TranslateLib => "TRANSLATE_LIB".to_string(),
        ConnectionAttribute::TranslateOption => "TRANSLATE_OPTION".to_string(),
        ConnectionAttribute::TxnIsolation => "TXN_ISOLATION".to_string(),
        ConnectionAttribute::CurrentCatalog => "CURRENT_CATALOG".to_string(),
        ConnectionAttribute::OdbcCursors => "ODBC_CURSORS".to_string(),
        ConnectionAttribute::QuietMode => "QUIET_MODE".to_string(),
        ConnectionAttribute::PacketSize => "PACKET_SIZE".to_string(),
        ConnectionAttribute::ConnectionTimeout => "CONNECTION_TIMEOUT".to_string(),
        ConnectionAttribute::DisconnectBehaviour => "DISCONNECT_BEHAVIOUR".to_string(),
        ConnectionAttribute::AsyncDbcFunctionsEnable => "ASYNC_DBC_FUNCTIONS_ENABLE".to_string(),
        ConnectionAttribute::AsyncDbcEvent => "ASYNC_DBC_EVENT".to_string(),
        ConnectionAttribute::EnlistInDtc => "ENLIST_IN_DTC".to_string(),
        ConnectionAttribute::EnlistInXa => "ENLIST_IN_XA".to_string(),
        ConnectionAttribute::ConnectionDead => "CONNECTION_DEAD".to_string(),
        ConnectionAttribute::AutoIpd => "AUTO_IPD".to_string(),
        ConnectionAttribute::MetadataId => "METADATA_ID".to_string(),
    }
}

// TODO: SQL-1109
// pub(crate) fn format_version(major: &str, minor: &str, patch: &str) -> String {
//     format!(
//         "{}.{}.{}",
//         format_version_part(major, 2),
//         format_version_part(minor, 2),
//         format_version_part(patch, 4)
//     )
// }
//
// fn format_version_part(part: &str, len: usize) -> String {
//     if len < part.len() {
//         return part.to_string();
//     }
//     format!("{}{}", "0".repeat(len - part.len()), part)
// }
//
// mod unit {
//     #[cfg(test)]
//     use super::format_version;
//
//     macro_rules! format_version_test {
//         ($func_name:ident, expected = $expected:expr, major = $major:expr, minor = $minor:expr, patch = $patch:expr) => {
//             #[test]
//             fn $func_name() {
//                 let actual = format_version($major, $minor, $patch);
//                 assert_eq!($expected, actual)
//             }
//         };
//     }
//
//     format_version_test!(
//         no_padding_needed,
//         expected = "10.11.1213",
//         major = "10",
//         minor = "11",
//         patch = "1213"
//     );
//
//     format_version_test!(
//         padding_needed,
//         expected = "01.01.0001",
//         major = "1",
//         minor = "1",
//         patch = "1"
//     );
//
//     format_version_test!(
//         parts_larger_than_length,
//         expected = "111.222.33333",
//         major = "111",
//         minor = "222",
//         patch = "33333"
//     );
// }
