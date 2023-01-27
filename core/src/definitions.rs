use num_derive::FromPrimitive;

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum SqlDataType {
    UNKNOWN_TYPE = 0,
    // also called SQL_VARIANT_TYPE since odbc 4.0
    CHAR = 1,
    NUMERIC = 2,
    DECIMAL = 3,
    /// Exact numeric value with precision 10 and scale 0 (signed: `-2[31] <= n <= 2[31] - 1`,
    /// unsigned: `0 <= n <= 2[32] - 1`).  An application uses `SQLGetTypeInfo` or `SQLColAttribute`
    /// to determine whether a particular data type or a particular column in a result set is
    /// unsigned.
    INTEGER = 4,
    SMALLINT = 5,
    FLOAT = 6,
    REAL = 7,
    /// Signed, approximate, numeric value with a binary precision 53 (zero or absolute value
    /// `10[-308]` to `10[308]`).
    DOUBLE = 8,
    DATETIME = 9,
    VARCHAR = 12,
    #[cfg(feature = "odbc_version_4")]
    UDT = 17,
    #[cfg(feature = "odbc_version_4")]
    ROW = 19,
    #[cfg(feature = "odbc_version_4")]
    ARRAY = 50,
    #[cfg(feature = "odbc_version_4")]
    MULTISET = 55,

    // one-parameter shortcuts for date/time data types
    DATE = 91,
    TIME = 92,
    /// Year, month, day, hour, minute, and second fields, with valid values as defined for the DATE
    /// and TIME data types.
    TIMESTAMP = 93,
    #[cfg(feature = "odbc_version_4")]
    TIME_WITH_TIMEZONE = 94,
    #[cfg(feature = "odbc_version_4")]
    TIMESTAMP_WITH_TIMEZONE = 95,

    // additional spec types: https://learn.microsoft.com/en-us/sql/odbc/reference/appendixes/sql-data-types?view=sql-server-ver16
    INTERVAL_YEAR = 101,
    INTERVAL_MONTH = 102,
    INTERVAL_DAY = 103,
    INTERVAL_HOUR = 104,
    INTERVAL_MINUTE = 105,
    INTERVAL_SECOND = 106,
    INTERVAL_YEAR_TO_MONTH = 107,
    INTERVAL_DAY_TO_HOUR = 108,
    INTERVAL_DAY_TO_MINUTE = 109,
    INTERVAL_DAY_TO_SECOND = 110,
    INTERVAL_HOUR_TO_MINUTE = 111,
    INTERVAL_HOUR_TO_SECOND = 112,
    INTERVAL_MINUTE_TO_SECOND = 113,

    // SQL extended datatypes:
    EXT_TIME_OR_INTERVAL = 10,
    EXT_TIMESTAMP = 11,
    EXT_LONG_VARCHAR = -1,
    EXT_BINARY = -2,
    EXT_VAR_BINARY = -3,
    EXT_LONG_VAR_BINARY = -4,
    EXT_BIG_INT = -5,
    EXT_TINY_INT = -6,
    EXT_BIT = -7,
    EXT_W_CHAR = -8,
    EXT_W_VARCHAR = -9,
    EXT_W_LONG_VARCHAR = -10,
    EXT_GUID = -11,
}
