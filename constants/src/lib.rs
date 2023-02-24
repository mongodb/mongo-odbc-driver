pub const VENDOR_IDENTIFIER: &str = "MongoDB";
pub const DRIVER_NAME: &str = "MongoDB Atlas SQL interface ODBC Driver";
pub const DBMS_NAME: &str = "MongoDB Atlas";
pub const ODBC_VERSION: &str = "03.80";
pub const DEFAULT_APP_NAME: &str = "odbc-driver";

// SQL states
pub const NOT_IMPLEMENTED: &str = "HYC00";
pub const TIMEOUT_EXPIRED: &str = "HYT00";
pub const GENERAL_ERROR: &str = "HY000";
pub const PROGRAM_TYPE_OUT_OF_RANGE: &str = "HY003";
pub const INVALID_SQL_TYPE: &str = "HY004";
pub const INVALID_ATTR_VALUE: &str = "HY024";
pub const INVALID_INFO_TYPE_VALUE: &str = "HY096";
pub const NO_DSN_OR_DRIVER: &str = "IM007";
pub const RIGHT_TRUNCATED: &str = "01004";
pub const OPTION_CHANGED: &str = "01S02";
pub const FRACTIONAL_TRUNCATION: &str = "01S07";
pub const UNABLE_TO_CONNECT: &str = "08001";
pub const INVALID_DESCRIPTOR_INDEX: &str = "07009";
pub const NO_RESULTSET: &str = "07005";
pub const RESTRICTED_DATATYPE: &str = "07006";
pub const INVALID_CURSOR_STATE: &str = "24000";
pub const FUNCTION_SEQUENCE_ERROR: &str = "HY010";
pub const UNSUPPORTED_FIELD_DESCRIPTOR: &str = "HY091";
pub const INVALID_ATTRIBUTE_OR_OPTION_IDENTIFIER: &str = "HY092";
pub const INDICATOR_VARIABLE_REQUIRED: &str = "22002";
pub const INTEGRAL_TRUNCATION: &str = "22003";
pub const INVALID_DATETIME_FORMAT: &str = "22007";
pub const INVALID_CHARACTER_VALUE: &str = "22018";

pub const SQL_ALL_TABLE_TYPES: &str = "%";
pub const SQL_ALL_CATALOGS: &str = "%";
pub const SQL_ALL_SCHEMAS: &str = "%";
