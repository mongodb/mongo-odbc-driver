use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum SqlBool {
    False = 0,
    True,
}

// Environment attributes

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum OdbcVersion {
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

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InfoType {
    // SQL_DRIVER_NAME
    DriverName = 6,
    // SQL_DRIVER_VER
    DriverVer = 7,
    // SQL_DRIVER_ODBC_VER
    DriverOdbcVer = 10,
    // SQL_SEARCH_PATTERN_ESCAPE
    SearchPatternEscape = 14,
    // SQL_DBMS_NAME
    DbmsName = 17,
    // SQL_DBMS_VER
    DbmsVer = 18,
    // SQL_CONCAT_NULL_BEHAVIOR
    ConcatNullBehavior = 22,
    // SQL_DATA_SOURCE_READ_ONLY
    DataSourceReadOnly = 25,
    // SQL_IDENTIFIER_QUOTE_CHAR
    IdentifierQuoteChar = 29,
    // SQL_OWNER_TERM
    OwnerTerm = 39,
    // SQL_CATALOG_NAME_SEPARATOR
    CatalogNameSeparator = 41,
    // SQL_CATALOG_TERM
    CatalogTerm = 42,
    // SQL_CONVERT_FUNCTIONS
    ConvertFunctions = 48,
    // SQL_NUMERIC_FUNCTIONS
    NumericFunctions = 49,
    // SQL_STRING_FUNCTIONS
    StringFunctions = 50,
    // SQL_SYSTEM_FUNCTIONS
    SystemFunctions = 51,
    // SQL_TIMEDATE_FUNCTIONS
    TimedateFunctions = 52,
    // SQL_CONVERT_BIGINT
    ConvertBigInt = 53,
    // SQL_CONVERT_BINARY
    ConvertBinary = 54,
    // SQL_CONVERT_BIT
    ConvertBit = 55,
    // SQL_CONVERT_CHAR
    ConvertChar = 56,
    // SQL_CONVERT_DATE
    ConvertDate = 57,
    // SQL_CONVERT_DECIMAL
    ConvertDecimal = 58,
    // SQL_CONVERT_DOUBLE
    ConvertDouble = 59,
    // SQL_CONVERT_FLOAT
    ConvertFloat = 60,
    // SQL_CONVERT_INTEGER
    ConvertInteger = 61,
    // SQL_CONVERT_LONGVARCHAR
    ConvertLongVarChar = 62,
    // SQL_CONVERT_NUMERIC
    ConvertNumeric = 63,
    // SQL_CONVERT_REAL
    ConvertReal = 64,
    // SQL_CONVERT_SMALLINT
    ConvertSmallInt = 65,
    // SQL_CONVERT_TIME
    ConvertTime = 66,
    // SQL_CONVERT_TIMESTAMP
    ConvertTimestamp = 67,
    // SQL_CONVERT_TINYINT
    ConvertTinyInt = 68,
    // SQL_CONVERT_VARBINARY
    ConvertVarBinary = 69,
    // SQL_CONVERT_VARCHAR
    ConvertVarChar = 70,
    // SQL_CONVERT_LONGVARBINARY
    ConvertLongVarBinary = 71,
    // SQL_GETDATA_EXTENSIONS
    GetDataExtensions = 81,
    // SQL_COLUMN_ALIAS
    ColumnAlias = 87,
    // SQL_GROUP_BY
    GroupBy = 88,
    // SQL_ORDER_BY_COLUMNS_IN_SELECT
    OrderByColumnsInSelect = 90,
    // SQL_OWNER_USAGE
    OwnerUsage = 91,
    // SQL_CATALOG_USAGE
    CatalogUsage = 92,
    // SQL_SPECIAL_CHARACTERS
    SpecialCharacters = 94,
    // SQL_MAX_COLUMNS_IN_GROUP_BY
    MaxColumnsInGroupBy = 97,
    // SQL_MAX_COLUMNS_IN_ORDER_BY
    MaxColumnsInOrderBy = 99,
    // SQL_MAX_COLUMNS_IN_SELECT
    MaxColumnsInSelect = 100,
    // SQL_TIMEDATE_ADD_INTERVALS
    TimedateAddIntervals = 109,
    // SQL_TIMEDATE_DIFF_INTERVALS
    TimedateDiffIntervals = 110,
    // SQL_CATALOG_LOCATION
    CatalogLocation = 114,
    // SQL_SQL_CONFORMANCE
    SqlConformance = 118,
    // SQL_CONVERT_WCHAR
    ConvertWChar = 122,
    // SQL_CONVERT_WLONGVARCHAR
    ConvertWLongVarChar = 125,
    // SQL_CONVERT_WVARCHAR
    ConvertWVarChar = 126,
    // SQL_ODBC_INTERFACE_CONFORMANCE
    OdbcInterfaceConformance = 152,
    // SQL_SQL92_PREDICATES
    Sql92Predicates = 160,
    // SQL_SQL92_RELATIONAL_JOIN_OPERATORS
    Sql92RelationalJoinOperators = 161,
    // SQL_AGGREGATE_FUNCTIONS
    AggregateFunctions = 169,
    // SQL_CONVERT_GUID
    ConvertGuid = 173,
    // SQL_RETURN_ESCAPE_CLAUSE
    ReturnEscapeClause = 180,
    // SQL_CATALOG_NAME
    CatalogName = 10003,
    // SQL_MAX_IDENTIFIER_LEN
    MaxIdentifierLen = 10005,
}

pub const SQL_CB_NULL: u16 = 0x0000;
pub const SQL_U16_ZERO: u16 = SQL_CB_NULL;
pub const SQL_CL_START: u16 = 0x0001;
pub const SQL_U32_ZERO: u32 = 0;
pub const SQL_OIC_CORE: u32 = 0x00000001;
pub const SQL_SC_SQL92_ENTRY: u32 = 0x00000001;
pub const SQL_INFO_Y: &str = "Y";
pub const SQL_GB_GROUP_BY_CONTAINS_SELECT: u16 = 0x0002;

// SQL_CONVERT_FUNCTIONS bitmask
pub const SQL_FN_CVT_CAST: u32 = 0x00000002;

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
pub const SQL_FN_STR_CONCAT: u32 = 0x00000001;
pub const SQL_FN_STR_LENGTH: u32 = 0x00000010;
pub const SQL_FN_STR_SUBSTRING: u32 = 0x00000800;
pub const SQL_FN_STR_BIT_LENGTH: u32 = 0x00080000;
pub const SQL_FN_STR_CHAR_LENGTH: u32 = 0x00100000;
pub const SQL_FN_STR_CHARACTER_LENGTH: u32 = 0x00200000;
pub const SQL_FN_STR_OCTET_LENGTH: u32 = 0x00400000;
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
