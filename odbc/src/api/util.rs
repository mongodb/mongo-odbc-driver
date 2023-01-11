use odbc_sys::Integer;

pub(crate) fn connection_attribute_to_string(attr: Integer) -> String {
    match attr {
        4 => "ASYNC_ENABLE".to_string(),
        101 => "ACCESS_MODE".to_string(),
        102 => "AUTO_COMMIT".to_string(),
        103 => "LOGIN_TIMEOUT".to_string(),
        104 => "TRACE".to_string(),
        105 => "TRACE_FILE".to_string(),
        106 => "TRANSLATE_LIB".to_string(),
        107 => "TRANSLATE_OPTION".to_string(),
        108 => "TXN_ISOLATION".to_string(),
        109 => "CURRENT_CATALOG".to_string(),
        110 => "ODBC_CURSORS".to_string(),
        111 => "QUIET_MODE".to_string(),
        112 => "PACKET_SIZE".to_string(),
        113 => "CONNECTION_TIMEOUT".to_string(),
        114 => "DISCONNECT_BEHAVIOUR".to_string(),
        117 => "ASYNC_DBC_FUNCTIONS_ENABLE".to_string(),
        119 => "ASYNC_DBC_EVENT".to_string(),
        1207 => "ENLIST_IN_DTC".to_string(),
        1208 => "ENLIST_IN_XA".to_string(),
        1209 => "CONNECTION_DEAD".to_string(),
        10001 => "AUTO_IPD".to_string(),
        10014 => "METADATA_ID".to_string(),
        _ => format!("<unknown: {}>", attr),
    }
}

pub(crate) fn environment_attribute_to_string(attr: Integer) -> String {
    match attr {
        200 => "OdbcVersion".to_string(),
        201 => "ConnectionPooling".to_string(),
        202 => "CpMatch".to_string(),
        10001 => "OutputNts".to_string(),
        _ => format!("<unknown: {}>", attr),
    }
}

pub(crate) fn statement_attribute_to_string(attr: Integer) -> String {
    match attr {
        10010 => "AppRowDesc".to_string(),
        10011 => "AppParamDesc".to_string(),
        10012 => "ImpRowDesc".to_string(),
        10013 => "ImpParamDesc".to_string(),
        -1 => "CursorScrollable".to_string(),
        -2 => "CursorSensitivity".to_string(),
        4 => "AsyncEnable".to_string(),
        7 => "Concurrency".to_string(),
        6 => "CursorType".to_string(),
        15 => "EnableAutoIpd".to_string(),
        16 => "FetchBookmarkPtr".to_string(),
        8 => "KeysetSize".to_string(),
        3 => "MaxLength".to_string(),
        1 => "MaxRows".to_string(),
        2 => "NoScan".to_string(),
        17 => "ParamBindOffsetPtr".to_string(),
        18 => "ParamBindType".to_string(),
        19 => "ParamOpterationPtr".to_string(),
        20 => "ParamStatusPtr".to_string(),
        21 => "ParamsProcessedPtr".to_string(),
        22 => "ParamsetSize".to_string(),
        0 => "QueryTimeout".to_string(),
        11 => "RetrieveData".to_string(),
        23 => "RowBindOffsetPtr".to_string(),
        5 => "RowBindType".to_string(),
        14 => "RowNumber".to_string(),
        24 => "RowOperationPtr".to_string(),
        25 => "RowStatusPtr".to_string(),
        26 => "RowsFetchedPtr".to_string(),
        27 => "RowArraySize".to_string(),
        10 => "SimulateCursor".to_string(),
        12 => "UseBookmarks".to_string(),
        29 => "AsyncStmtEvent".to_string(),
        10014 => "MetadataId".to_string(),
        _ => format!("<unknown: {}>", attr),
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
