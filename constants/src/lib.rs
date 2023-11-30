use lazy_static::lazy_static;

pub const VENDOR_IDENTIFIER: &str = "MongoDB";
pub const DRIVER_NAME: &str = "MongoDB Atlas SQL ODBC Driver";
pub const DBMS_NAME: &str = "MongoDB Atlas";
pub const ODBC_VERSION: &str = "03.80";
pub const DRIVER_SHORT_NAME: &str = "mongodb-odbc";

lazy_static! {
    pub static ref DRIVER_METRICS_VERSION: String = format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );
    pub static ref DRIVER_LOG_VERSION: String = format!(
        "{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR")
    );
    pub static ref DEFAULT_APP_NAME: String =
        format!("{}+{}", DRIVER_SHORT_NAME, DRIVER_METRICS_VERSION.as_str());
    pub static ref DRIVER_ODBC_VERSION: String = format_driver_version();
}

// SQL states, stored as [ODBC2 state, ODBC3 state]
pub const NOT_IMPLEMENTED: [&str; 2] = ["S1C00", "HYC00"];
pub const TIMEOUT_EXPIRED: [&str; 2] = ["S1T00", "HYT00"];
pub const GENERAL_ERROR: [&str; 2] = ["S1000", "HY000"];
pub const PROGRAM_TYPE_OUT_OF_RANGE: [&str; 2] = ["S1003", "HY003"];
pub const INVALID_SQL_TYPE: [&str; 2] = ["S1004", "HY004"];
pub const INVALID_ATTR_VALUE: [&str; 2] = ["S1009", "HY024"];
pub const INVALID_INFO_TYPE_VALUE: [&str; 2] = ["S1096", "HY096"];
pub const NO_DSN_OR_DRIVER: [&str; 2] = ["IM007", "IM007"];
pub const GENERAL_WARNING: [&str; 2] = ["01000", "01000"];
pub const RIGHT_TRUNCATED: [&str; 2] = ["01004", "01004"];
pub const OPTION_CHANGED: [&str; 2] = ["01S02", "01S02"];
pub const FRACTIONAL_TRUNCATION: [&str; 2] = ["01S07", "01S07"];
pub const UNABLE_TO_CONNECT: [&str; 2] = ["08001", "08001"];
pub const INVALID_DESCRIPTOR_INDEX: [&str; 2] = ["07009", "07009"];
pub const NO_RESULTSET: [&str; 2] = ["24000", "07005"];
pub const RESTRICTED_DATATYPE: [&str; 2] = ["07006", "07006"];
pub const INVALID_CURSOR_STATE: [&str; 2] = ["24000", "24000"];
pub const FUNCTION_SEQUENCE_ERROR: [&str; 2] = ["S1010", "HY010"];
pub const UNSUPPORTED_FIELD_DESCRIPTOR: [&str; 2] = ["S1091", "HY091"];
pub const INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER: [&str; 2] = ["S1092", "HY092"];
pub const INDICATOR_VARIABLE_REQUIRED: [&str; 2] = ["22002", "22002"];
pub const INTEGRAL_TRUNCATION: [&str; 2] = ["22003", "22003"];
pub const INVALID_DATETIME_FORMAT: [&str; 2] = ["22008", "22007"];
pub const INVALID_CHARACTER_VALUE: [&str; 2] = ["22005", "22018"];
pub const CONNECTION_NOT_OPEN: [&str; 2] = ["08003", "08003"];

pub const SQL_ALL_TABLE_TYPES: &str = "%";
pub const SQL_ALL_CATALOGS: &str = "%";
pub const SQL_ALL_SCHEMAS: &str = "%";

fn format_driver_version() -> String {
    // The driver version can be obtained from the Cargo.toml file.
    // The env! macro call below gets the version from the Cargo file
    // at compile time.
    let version_major = env!("CARGO_PKG_VERSION_MAJOR");
    let version_minor = env!("CARGO_PKG_VERSION_MINOR");
    let version_patch = env!("CARGO_PKG_VERSION_PATCH");

    format_version(version_major, version_minor, version_patch)
}

fn format_version(major: &str, minor: &str, patch: &str) -> String {
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
        major = "0",
        minor = "1",
        patch = "0"
    );
}

#[cfg(test)]
mod version {
    use cargo_toml::Manifest;
    use std::{env, path::PathBuf, process::Command};

    fn get_workspace_root() -> anyhow::Result<PathBuf> {
        let current_dir = env::current_dir()?;
        let cmd_output = Command::new("cargo")
            .args(["metadata", "--format-version=1"])
            .output()?;

        if !cmd_output.status.success() {
            return Ok(current_dir);
        }

        let json = serde_json::from_str::<serde_json::Value>(
            String::from_utf8(cmd_output.stdout)?.as_str(),
        )?;
        let path = match json.get("workspace_root") {
            Some(val) => match val.as_str() {
                Some(val) => val,
                None => return Ok(current_dir),
            },
            None => return Ok(current_dir),
        };
        Ok(PathBuf::from(path))
    }

    fn get_member_versions() -> Vec<(String, String)> {
        let workspace = get_workspace_root().unwrap();
        let mut path = workspace.clone();
        path.push("Cargo.toml");
        let manifest = Manifest::from_path(path).unwrap();
        let members = manifest.workspace.unwrap().members;
        let mut versions = Vec::new();
        for member in members {
            let mut path = workspace.clone();
            path.push(&member);
            path.push("Cargo.toml");
            let manifest = Manifest::from_path(path).unwrap();
            versions.push((member, manifest.package.unwrap().version.unwrap()));
        }
        versions
    }

    #[test]
    fn ensure_version_sync() {
        let members = get_member_versions();
        if members.is_empty() {
            panic!("didn't get any workspace members")
        }
        assert!(
            members.windows(2).all(|w| w[0].1 == w[1].1),
            "not all versions are equal: {members:?}"
        )
    }
}
