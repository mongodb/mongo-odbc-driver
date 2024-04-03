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

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct OdbcState<'a> {
    pub odbc_2_state: &'a str,
    pub odbc_3_state: &'a str,
}

pub const NOT_IMPLEMENTED: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1C00",
    odbc_3_state: "HYC00",
};
pub const TIMEOUT_EXPIRED: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1T00",
    odbc_3_state: "HYT00",
};
pub const GENERAL_ERROR: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1000",
    odbc_3_state: "HY000",
};
pub const PROGRAM_TYPE_OUT_OF_RANGE: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1003",
    odbc_3_state: "HY003",
};
pub const INVALID_SQL_TYPE: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1004",
    odbc_3_state: "HY004",
};
pub const OPERATION_CANCELLED: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1008",
    odbc_3_state: "HY008",
};
pub const INVALID_ATTR_VALUE: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1009",
    odbc_3_state: "HY024",
};
pub const INVALID_INFO_TYPE_VALUE: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1096",
    odbc_3_state: "HY096",
};
pub const INVALID_DRIVER_COMPLETION: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1110",
    odbc_3_state: "HY110",
};
pub const NO_DSN_OR_DRIVER: OdbcState<'static> = OdbcState {
    odbc_2_state: "IM007",
    odbc_3_state: "IM007",
};
pub const GENERAL_WARNING: OdbcState<'static> = OdbcState {
    odbc_2_state: "01000",
    odbc_3_state: "01000",
};
pub const RIGHT_TRUNCATED: OdbcState<'static> = OdbcState {
    odbc_2_state: "01004",
    odbc_3_state: "01004",
};
pub const OPTION_CHANGED: OdbcState<'static> = OdbcState {
    odbc_2_state: "01S02",
    odbc_3_state: "01S02",
};
pub const FRACTIONAL_TRUNCATION: OdbcState<'static> = OdbcState {
    odbc_2_state: "01S07",
    odbc_3_state: "01S07",
};
pub const UNABLE_TO_CONNECT: OdbcState<'static> = OdbcState {
    odbc_2_state: "08001",
    odbc_3_state: "08001",
};
pub const INVALID_DESCRIPTOR_INDEX: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1002",
    odbc_3_state: "07009",
};

// ODBC 3.x SQLSTATE 07009 is mapped to ODBC 2.x SQLSTATE S1093 if the underlying function is SQLBindParameter or SQLDescribeParam.
// S1093 is a DM error for SQLBindParameter, but not always for SQLDescribeParam.
// If we implement the latter, we will need to add that error here.

pub const INVALID_COLUMN_NUMBER: OdbcState<'static> = OdbcState {
    odbc_2_state: "07009",
    odbc_3_state: "07009",
};
pub const NO_RESULTSET: OdbcState<'static> = OdbcState {
    odbc_2_state: "24000",
    odbc_3_state: "07005",
};
pub const RESTRICTED_DATATYPE: OdbcState<'static> = OdbcState {
    odbc_2_state: "07006",
    odbc_3_state: "07006",
};
pub const INVALID_CURSOR_STATE: OdbcState<'static> = OdbcState {
    odbc_2_state: "24000",
    odbc_3_state: "24000",
};
pub const FUNCTION_SEQUENCE_ERROR: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1010",
    odbc_3_state: "HY010",
};
pub const INVALID_FIELD_DESCRIPTOR: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1091",
    odbc_3_state: "HY091",
};
pub const INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1092",
    odbc_3_state: "HY092",
};
pub const FETCH_TYPE_OUT_OF_RANGE: OdbcState<'static> = OdbcState {
    odbc_2_state: "S1106",
    odbc_3_state: "HY106",
};
pub const INDICATOR_VARIABLE_REQUIRED: OdbcState<'static> = OdbcState {
    odbc_2_state: "22002",
    odbc_3_state: "22002",
};
pub const INTEGRAL_TRUNCATION: OdbcState<'static> = OdbcState {
    odbc_2_state: "22003",
    odbc_3_state: "22003",
};
pub const INVALID_DATETIME_FORMAT: OdbcState<'static> = OdbcState {
    odbc_2_state: "22008",
    odbc_3_state: "22007",
};
pub const INVALID_CHARACTER_VALUE: OdbcState<'static> = OdbcState {
    odbc_2_state: "22005",
    odbc_3_state: "22018",
};
pub const CONNECTION_NOT_OPEN: OdbcState<'static> = OdbcState {
    odbc_2_state: "08003",
    odbc_3_state: "08003",
};

pub const SQL_ALL_TABLE_TYPES: &str = "%";
pub const SQL_ALL_CATALOGS: &str = "%";
pub const SQL_ALL_SCHEMAS: &str = "%";

pub const SQL_CB_NULL: u16 = 0x0000;
pub const MAX_COLUMNS_U16_ZERO: u16 = 0x0000;
pub const SQL_CL_START: u16 = 0x0001;
pub const MAX_COLUMNS_U32_ZERO: u32 = 0x0;
pub const SQL_OIC_CORE: u32 = 0x00000001;
pub const SQL_SC_SQL92_ENTRY: u32 = 0x00000001;
pub const COLUMN_ALIAS_INFO_Y: &str = "Y";
pub const SQL_GB_GROUP_BY_CONTAINS_SELECT: u16 = 0x0002;
pub const SQL_CB_PRESERVE: u16 = 2;
pub const SQL_CA1_NEXT: u32 = 0x00000001;
pub const SQL_CA2_READ_ONLY_CONCURRENCY: u32 = 0x00000001;
#[allow(unused)]
pub const SQL_CA2_MAX_ROWS_SELECT: u32 = 0x00000080;
pub const SQL_CA2_CRC_EXACT: u32 = 0x00001000;
pub const MONGO_CA2_SUPPORT: u32 = SQL_CA2_CRC_EXACT | SQL_CA2_READ_ONLY_CONCURRENCY;
pub const SQL_SO_FORWARD_ONLY: u32 = 0x00000001;
pub const SQL_SO_STATIC: u32 = 0x00000010;
pub const MONGO_SO_SUPPORT: u32 = SQL_SO_FORWARD_ONLY | SQL_SO_STATIC;
pub const SQL_TXN_SERIALIZABLE: u32 = 0x00000008;
pub const SQL_SCCO_READ_ONLY: u32 = 0x00000001;
pub const SQL_LCK_NO_CHANGE: u32 = 0x00000001;

// SQL_CONVERT_FUNCTIONS bitmask
pub const SQL_FN_CVT_CAST: u32 = 0x00000002;

// BitMask for supported CAST Types
pub const SQL_CVT_CHAR: u32 = 0x00000001;
pub const SQL_CVT_NUMERIC: u32 = 0x00000002;
#[allow(unused)]
pub const SQL_CVT_DECIMAL: u32 = 0x00000004;
pub const SQL_CVT_INTEGER: u32 = 0x00000008;
pub const SQL_CVT_SMALLINT: u32 = 0x00000010;
pub const SQL_CVT_FLOAT: u32 = 0x00000020;
pub const SQL_CVT_REAL: u32 = 0x00000040;
pub const SQL_CVT_DOUBLE: u32 = 0x00000080;
pub const SQL_CVT_VARCHAR: u32 = 0x00000100;
#[allow(unused)]
pub const SQL_CVT_LONGVARCHAR: u32 = 0x00000200;
#[allow(unused)]
pub const SQL_CVT_BINARY: u32 = 0x00000400;
#[allow(unused)]
pub const SQL_CVT_VARBINARY: u32 = 0x00000800;
pub const SQL_CVT_BIT: u32 = 0x00001000;
#[allow(unused)]
pub const SQL_CVT_TINYINT: u32 = 0x00002000;
#[allow(unused)]
pub const SQL_CVT_BIGINT: u32 = 0x00004000;
#[allow(unused)]
pub const SQL_CVT_DATE: u32 = 0x00008000;
#[allow(unused)]
pub const SQL_CVT_TIME: u32 = 0x00010000;
pub const SQL_CVT_TIMESTAMP: u32 = 0x00020000;
#[allow(unused)]
pub const SQL_CVT_LONGVARBINARY: u32 = 0x00040000;
#[allow(unused)]
pub const SQL_CVT_INTERVAL_YEAR_MONTH: u32 = 0x00080000;
#[allow(unused)]
pub const SQL_CVT_INTERVAL_DAY_TIME: u32 = 0x00100000;
#[allow(unused)]
pub const SQL_CVT_WCHAR: u32 = 0x00200000;
#[allow(unused)]
pub const SQL_CVT_WLONGVARCHAR: u32 = 0x00400000;
#[allow(unused)]
pub const SQL_CVT_WVARCHAR: u32 = 0x00800000;
#[allow(unused)]
pub const SQL_CVT_GUID: u32 = 0x01000000;
pub const MONGO_CAST_SUPPORT: u32 = SQL_CVT_CHAR
    | SQL_CVT_NUMERIC
    | SQL_CVT_INTEGER
    | SQL_CVT_SMALLINT
    | SQL_CVT_FLOAT
    | SQL_CVT_REAL
    | SQL_CVT_DOUBLE
    | SQL_CVT_VARCHAR
    | SQL_CVT_BIT
    | SQL_CVT_TIMESTAMP;

// SQL_NUMERIC_FUNCTIONS bitmasks
pub const SQL_FN_NUM_ABS: u32 = 0x00000001;
pub const SQL_FN_NUM_CEILING: u32 = 0x00000020;
pub const SQL_FN_NUM_COS: u32 = 0x00000040;
pub const SQL_FN_NUM_FLOOR: u32 = 0x00000200;
pub const SQL_FN_NUM_LOG: u32 = 0x00000400;
pub const SQL_FN_NUM_LOG10: u32 = 0x00080000;
pub const SQL_FN_NUM_MOD: u32 = 0x00000800;
pub const SQL_FN_NUM_SIN: u32 = 0x00002000;
pub const SQL_FN_NUM_SQRT: u32 = 0x00004000;
pub const SQL_FN_NUM_TAN: u32 = 0x00008000;
pub const SQL_FN_NUM_DEGREES: u32 = 0x00040000;
pub const SQL_FN_NUM_POWER: u32 = 0x00100000;
pub const SQL_FN_NUM_RADIANS: u32 = 0x00200000;
pub const SQL_FN_NUM_ROUND: u32 = 0x00400000;

pub const SQL_FN_TD_NOW: u32 = 0x00000001;
pub const SQL_FN_TD_CURDATE: u32 = 0x00000002;
pub const SQL_FN_TD_DAYOFMONTH: u32 = 0x00000004;
pub const SQL_FN_TD_DAYOFWEEK: u32 = 0x00000008;
pub const SQL_FN_TD_DAYOFYEAR: u32 = 0x00000010;
pub const SQL_FN_TD_MONTH: u32 = 0x00000020;
pub const SQL_FN_TD_QUARTER: u32 = 0x00000040;
pub const SQL_FN_TD_WEEK: u32 = 0x00000080;
pub const SQL_FN_TD_YEAR: u32 = 0x00000100;
pub const SQL_FN_TD_CURTIME: u32 = 0x00000200;
pub const SQL_FN_TD_HOUR: u32 = 0x00000400;
pub const SQL_FN_TD_MINUTE: u32 = 0x00000800;
pub const SQL_FN_TD_SECOND: u32 = 0x00001000;
pub const SQL_FN_TD_TIMESTAMPADD: u32 = 0x00002000;
pub const SQL_FN_TD_TIMESTAMPDIFF: u32 = 0x00004000;
pub const SQL_FN_TD_DAYNAME: u32 = 0x00008000;
pub const SQL_FN_TD_MONTHNAME: u32 = 0x00010000;
pub const SQL_FN_TD_CURRENT_DATE: u32 = 0x00020000;
pub const SQL_FN_TD_CURRENT_TIME: u32 = 0x00040000;

// SQL_STRING_FUNCTIONS bitmasks
#[allow(unused)]
pub const SQL_FN_STR_CONCAT: u32 = 0x00000001;
#[allow(unused)]
pub const SQL_FN_STR_INSERT: u32 = 0x00000002;
#[allow(unused)]
pub const SQL_FN_STR_LEFT: u32 = 0x00000004;
#[allow(unused)]
pub const SQL_FN_STR_LTRIM: u32 = 0x00000008;
#[allow(unused)]
pub const SQL_FN_STR_LENGTH: u32 = 0x00000010;
#[allow(unused)]
pub const SQL_FN_STR_LOCATE: u32 = 0x00000020;
#[allow(unused)]
pub const SQL_FN_STR_LCASE: u32 = 0x00000040;
#[allow(unused)]
pub const SQL_FN_STR_REPEAT: u32 = 0x00000080;
#[allow(unused)]
pub const SQL_FN_STR_REPLACE: u32 = 0x00000100;
#[allow(unused)]
pub const SQL_FN_STR_SUBSTRING: u32 = 0x00000800;
#[allow(unused)]
pub const SQL_FN_STR_UCASE: u32 = 0x00001000;
#[allow(unused)]
pub const SQL_FN_STR_ASCII: u32 = 0x00002000;
#[allow(unused)]
pub const SQL_FN_STR_CHAR: u32 = 0x00004000;
#[allow(unused)]
pub const SQL_FN_STR_DIFFERENCE: u32 = 0x00008000;
#[allow(unused)]
pub const SQL_FN_STR_LOCATE_2: u32 = 0x00010000;
#[allow(unused)]
pub const SQL_FN_STR_SOUNDEX: u32 = 0x00020000;
#[allow(unused)]
pub const SQL_FN_STR_SPACE: u32 = 0x00040000;
#[allow(unused)]
pub const SQL_FN_STR_BIT_LENGTH: u32 = 0x00080000;
#[allow(unused)]
pub const SQL_FN_STR_CHAR_LENGTH: u32 = 0x00100000;
#[allow(unused)]
pub const SQL_FN_STR_CHARACTER_LENGTH: u32 = 0x00200000;
#[allow(unused)]
pub const SQL_FN_STR_OCTET_LENGTH: u32 = 0x00400000;
#[allow(unused)]
pub const SQL_FN_STR_POSITION: u32 = 0x00800000;

// SQL_TIMEDATE_FUNCTIONS functions
pub const SQL_FN_TD_CURRENT_TIMESTAMP: u32 = 0x00080000;
pub const SQL_FN_TD_EXTRACT: u32 = 0x00100000;

// SQL_CATALOG_USAGE bitmasks
pub const SQL_CU_DML_STATEMENTS: u32 = 0x00000001;

// SQL_GETDATA_EXTENSIONS bitmasks
pub const SQL_GD_ANY_COLUMN: u32 = 0x00000001;
pub const SQL_GD_ANY_ORDER: u32 = 0x00000002;

// SQL_TIMEDATE_ADD_INTERVALS and SQL_TIMEDATE_DIFF_INTERVALS functions
pub const SQL_FN_TSI_FRAC_SECOND: u32 = 0x00000001;
pub const SQL_FN_TSI_SECOND: u32 = 0x00000002;
pub const SQL_FN_TSI_MINUTE: u32 = 0x00000004;
pub const SQL_FN_TSI_HOUR: u32 = 0x00000008;
pub const SQL_FN_TSI_DAY: u32 = 0x00000010;
pub const SQL_FN_TSI_WEEK: u32 = 0x00000020;
pub const SQL_FN_TSI_MONTH: u32 = 0x00000040;
pub const SQL_FN_TSI_QUARTER: u32 = 0x00000080;
pub const SQL_FN_TSI_YEAR: u32 = 0x00000100;

// SQL_SQL92_PREDICATES bitmasks
pub const SQL_SP_EXISTS: u32 = 0x00000001;
pub const SQL_SP_ISNOTNULL: u32 = 0x00000002;
pub const SQL_SP_ISNULL: u32 = 0x00000004;
pub const SQL_SP_LIKE: u32 = 0x00000200;
pub const SQL_SP_IN: u32 = 0x00000400;
pub const SQL_SP_BETWEEN: u32 = 0x00000800;
pub const SQL_SP_COMPARISON: u32 = 0x00001000;
pub const SQL_SP_QUANTIFIED_COMPARISON: u32 = 0x00002000;

// SQL_SQL92_RELATIONAL_JOIN_OPERATORS bitmasks
pub const SQL_SRJO_CROSS_JOIN: u32 = 0x00000002;
pub const SQL_SRJO_INNER_JOIN: u32 = 0x00000010;
pub const SQL_SRJO_LEFT_OUTER_JOIN: u32 = 0x00000040;
pub const SQL_SRJO_RIGHT_OUTER_JOIN: u32 = 0x00000100;

// SQL_AGGREGATE_FUNCTIONS bitmasks
pub const SQL_AF_AVG: u32 = 0x00000001;
pub const SQL_AF_COUNT: u32 = 0x00000002;
pub const SQL_AF_MAX: u32 = 0x00000004;
pub const SQL_AF_MIN: u32 = 0x00000008;
pub const SQL_AF_SUM: u32 = 0x00000010;
pub const SQL_AF_DISTINCT: u32 = 0x00000020;
pub const SQL_AF_ALL: u32 = 0x00000040;

pub const SQL_OJ_LEFT: u32 = 0x00000001;
pub const SQL_OJ_NOT_ORDERED: u32 = 0x00000010;
pub const SQL_OJ_INNER: u32 = 0x00000020;
pub const SQL_OJ_ALL_COMPARISON_OPS: u32 = 0x00000040;

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
