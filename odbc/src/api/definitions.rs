use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum SqlBool {
    False = 0,
    True,
}

// Environment attributes
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum OdbcVersion {
    Odbc2 = 2,
    Odbc3 = 3,
    Odbc3_80 = 380,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum ConnectionPooling {
    Off = 0,
    OnePerDriver,
    OnePerHEnv,
    DriverAware,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum CpMatch {
    Strict = 0,
    Relaxed,
}

// Statement attributes

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorScrollable {
    NonScrollable = 0,
    Scrollable,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorSensitivity {
    Unspecified = 0,
    Insensitive,
    Sensitive,
}

#[derive(Clone, Copy, Debug)]
pub enum AsyncEnable {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Concurrency {
    ReadOnly = 1,
    Lock = 2,
    RowVer = 4,
    Values = 8,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorType {
    ForwardOnly = 0,
    KeysetDriven = -1,
    Dynamic = -2,
    Static = -3,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum NoScan {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug)]
pub enum BindType {
    BindByColumn = 0,
}

#[derive(Clone, Copy, Debug)]
pub enum ParamOperationPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamStatusPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamsProcessedPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamsetSize {}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum RetrieveData {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug)]
pub enum RowOperationPtr {}

#[derive(Clone, Copy, Debug)]
pub enum SimulateCursor {
    NonUnique = 0,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum UseBookmarks {
    Off = 0,
    Variable = 2,
}

#[derive(Clone, Copy, Debug)]
pub enum AsyncStmtEvent {}

#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum ConnectionAttribute {
    SQL_ATTR_ASYNC_ENABLE = 4,
    SQL_ATTR_ACCESS_MODE = 101,
    SQL_ATTR_AUTOCOMMIT = 102,
    SQL_ATTR_LOGIN_TIMEOUT = 103,
    SQL_ATTR_TRACE = 104,
    SQL_ATTR_TRACEFILE = 105,
    SQL_ATTR_TRANSLATE_LIB = 106,
    SQL_ATTR_TRANSLATE_OPTION = 107,
    SQL_ATTR_TXN_ISOLATION = 108,
    SQL_ATTR_CURRENT_CATALOG = 109,
    SQL_ATTR_ODBC_CURSORS = 110,
    SQL_ATTR_QUIET_MODE = 111,
    SQL_ATTR_PACKET_SIZE = 112,
    SQL_ATTR_CONNECTION_TIMEOUT = 113,
    SQL_ATTR_DISCONNECT_BEHAVIOR = 114,
    SQL_ATTR_ASYNC_DBC_FUNCTIONS_ENABLE = 117,
    SQL_ATTR_ASYNC_DBC_EVENT = 119,
    SQL_ATTR_ENLIST_IN_DTC = 1207,
    SQL_ATTR_ENLIST_IN_XA = 1208,
    SQL_ATTR_CONNECTION_DEAD = 1209,
    SQL_ATTR_APP_WCHAR_TYPE = 1061,
    SQL_ATTR_AUTO_IPD = 10001,
    SQL_ATTR_METADATA_ID = 10014,
}

#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum EnvironmentAttribute {
    SQL_ATTR_ODBC_VERSION = 200,
    SQL_ATTR_CONNECTION_POOLING = 201,
    SQL_ATTR_CP_MATCH = 202,
    SQL_ATTR_OUTPUT_NTS = 10001,
    SQL_ATTR_DRIVER_UNICODE_TYPE = 1065,
}

#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum StatementAttribute {
    SQL_ATTR_CURSOR_SCROLLABLE = -1,
    SQL_ATTR_CURSOR_SENSITIVITY = -2,
    SQL_ATTR_QUERY_TIMEOUT = 0,
    SQL_ATTR_MAX_ROWS = 1,
    SQL_ATTR_NOSCAN = 2,
    SQL_ATTR_MAX_LENGTH = 3,
    SQL_ATTR_ASYNC_ENABLE = 4,
    SQL_ATTR_ROW_BIND_TYPE = 5,
    SQL_ATTR_CURSOR_TYPE = 6,
    SQL_ATTR_CONCURRENCY = 7,
    SQL_ATTR_KEYSET_SIZE = 8,
    // Never renamed to SQL_ATTR_ROWSET_SIZE
    SQL_ROWSET_SIZE = 9,
    SQL_ATTR_SIMULATE_CURSOR = 10,
    SQL_ATTR_RETRIEVE_DATA = 11,
    SQL_ATTR_USE_BOOKMARKS = 12,
    // Also has no SQL_ATTR version
    SQL_GET_BOOKMARK = 13,
    SQL_ATTR_ROW_NUMBER = 14,
    SQL_ATTR_ENABLE_AUTO_IPD = 15,
    SQL_ATTR_FETCH_BOOKMARK_PTR = 16,
    SQL_ATTR_PARAM_BIND_OFFSET_PTR = 17,
    SQL_ATTR_PARAM_BIND_TYPE = 18,
    SQL_ATTR_PARAM_OPERATION_PTR = 19,
    SQL_ATTR_PARAM_STATUS_PTR = 20,
    SQL_ATTR_PARAMS_PROCESSED_PTR = 21,
    SQL_ATTR_PARAMSET_SIZE = 22,
    SQL_ATTR_ROW_BIND_OFFSET_PTR = 23,
    SQL_ATTR_ROW_OPERATION_PTR = 24,
    SQL_ATTR_ROW_STATUS_PTR = 25,
    SQL_ATTR_ROWS_FETCHED_PTR = 26,
    SQL_ATTR_ROW_ARRAY_SIZE = 27,
    // there is no 28, apparently
    SQL_ATTR_ASYNC_STMT_EVENT = 29,
    SQL_ATTR_SAMPLE_SIZE = 30,
    SQL_ATTR_DYNAMIC_COLUMNS = 31,
    SQL_ATTR_TYPE_EXCEPTION_BEHAVIOR = 32,
    SQL_ATTR_LENGTH_EXCEPTION_BEHAVIOR = 33,
    SQL_ATTR_APP_ROW_DESC = 10010,
    SQL_ATTR_APP_PARAM_DESC = 10011,
    SQL_ATTR_IMP_ROW_DESC = 10012,
    SQL_ATTR_IMP_PARAM_DESC = 10013,
    SQL_ATTR_METADATA_ID = 10014,
}

/// Extended C Types range 4000 and above. Range of -100 thru 200 is reserved by Driver Manager.
/// `SQL_C_TYPES_EXTENDED`.
pub const C_TYPES_EXTENDED: i16 = 0x04000;

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum CDataType {
    SQL_ARD_TYPE = -99,
    SQL_APD_TYPE = -100,
    SQL_C_UTINYINT = -28,
    SQL_C_UBIGINT = -27,
    SQL_C_STINYINT = -26,
    SQL_C_SBIGINT = -25,
    SQL_C_ULONG = -18,
    SQL_C_USHORT = -17,
    SQL_C_SLONG = -16,
    SQL_C_SSHORT = -15,
    SQL_C_GUID = -11,
    SQL_C_WCHAR = -8,
    SQL_C_BIT = -7,
    SQL_C_BINARY = -2,
    SQL_C_CHAR = 1,
    SQL_C_NUMERIC = 2,
    SQL_C_FLOAT = 7,
    SQL_C_DOUBLE = 8,
    SQL_C_DATE = 9,
    SQL_C_TIME = 10,
    SQL_C_TIMESTAMP = 11,
    SQL_C_TYPE_DATE = 91,
    SQL_C_TYPE_TIME = 92,
    SQL_C_TYPE_TIMESTAMP = 93,
    SQL_C_TYPE_TIME_WITH_TIMEZONE = 94,
    SQL_C_TYPE_TIMESTAMP_WITH_TIMEZONE = 95,
    SQL_C_DEFAULT = 99,
    SQL_C_INTERVAL_YEAR = 101,
    SQL_C_INTERVAL_MONTH = 102,
    SQL_C_INTERVAL_DAY = 103,
    SQL_C_INTERVAL_HOUR = 104,
    SQL_C_INTERVAL_MINUTE = 105,
    SQL_C_INTERVAL_SECOND = 106,
    SQL_C_INTERVAL_YEAR_TO_MONTH = 107,
    SQL_C_INTERVAL_DAY_TO_HOUR = 108,
    SQL_C_INTERVAL_DAY_TO_MINUTE = 109,
    SQL_C_INTERVAL_DAY_TO_SECOND = 110,
    SQL_C_INTERVAL_HOUR_TO_MINUTE = 111,
    SQL_C_INTERVAL_HOUR_TO_SECOND = 112,
    SQL_C_INTERVAL_MINUTE_TO_SECOND = 113,
    SQL_C_SS_TIME2 = C_TYPES_EXTENDED,
    SQL_C_SS_TIMESTAMPOFFSET = C_TYPES_EXTENDED + 1,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum DiagType {
    SQL_DIAG_RETURNCODE = 1,
    SQL_DIAG_NUMBER = 2,
    SQL_DIAG_ROW_COUNT = 3,
    SQL_DIAG_SQLSTATE = 4,
    SQL_DIAG_NATIVE = 5,
    SQL_DIAG_MESSAGE_TEXT = 6,
    SQL_DIAG_DYNAMIC_FUNCTION = 7,
    SQL_DIAG_CLASS_ORIGIN = 8,
    SQL_DIAG_SUBCLASS_ORIGIN = 9,
    SQL_DIAG_CONNECTION_NAME = 10,
    SQL_DIAG_SERVER_NAME = 11,
    SQL_DIAG_DYNAMIC_FUNCTION_CODE = 12,
    SQL_DIAG_CURSOR_ROW_COUNT = -1249,
    SQL_DIAG_ROW_NUMBER = -1248,
    SQL_DIAG_COLUMN_NUMBER = -1247,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum ParamType {
    SQL_PARAM_TYPE_UNKNOWN = 0,
    SQL_PARAM_INPUT = 1,
    SQL_PARAM_INPUT_OUTPUT = 2,
    SQL_RESULT_COL = 3,
    SQL_PARAM_OUTPUT = 4,
    SQL_RETURN_VALUE = 5,
    SQL_PARAM_INPUT_OUTPUT_STREAM = 8,
    SQL_PARAM_OUTPUT_STREAM = 16,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum BulkOperation {
    SQL_ADD = 4,
    SQL_UPDATE_BY_BOOKMARK = 5,
    SQL_DELETE_BY_BOOKMARK = 6,
    SQL_FETCH_BY_BOOKMARK = 7,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum CompletionType {
    SQL_COMMIT = 0,
    SQL_ROLLBACK = 1,
}

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum FetchOrientation {
    SQL_FETCH_NEXT = 1,
    SQL_FETCH_FIRST = 2,
    SQL_FETCH_LAST = 3,
    SQL_FETCH_PRIOR = 4,
    SQL_FETCH_ABSOLUTE = 5,
    SQL_FETCH_RELATIVE = 6,
    SQL_FETCH_FIRST_USER = 31,
    SQL_FETCH_FIRST_SYSTEM = 32,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum Nullability {
    SQL_NO_NULLS = 0,
    SQL_NULLABLE = 1,
    SQL_NULLABLE_UNKNOWN = 2,
}

pub const SQL_ROW_NUMBER_UNKNOWN: isize = -2;

pub const SQL_CB_NULL: u16 = 0x0000;
pub const SQL_U16_ZERO: u16 = 0x0000;
pub const SQL_CL_START: u16 = 0x0001;
pub const SQL_U32_ZERO: u32 = 0x0;
pub const SQL_OIC_CORE: u32 = 0x00000001;
pub const SQL_SC_SQL92_ENTRY: u32 = 0x00000001;
pub const SQL_INFO_Y: &str = "Y";
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
pub const SQL_FN_NUM_MOD: u32 = 0x00000800;
pub const SQL_FN_NUM_SIN: u32 = 0x00002000;
pub const SQL_FN_NUM_SQRT: u32 = 0x00004000;
pub const SQL_FN_NUM_TAN: u32 = 0x00008000;
pub const SQL_FN_NUM_DEGREES: u32 = 0x00040000;
pub const SQL_FN_NUM_POWER: u32 = 0x00100000;
pub const SQL_FN_NUM_RADIANS: u32 = 0x00200000;
pub const SQL_FN_NUM_ROUND: u32 = 0x00400000;

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
