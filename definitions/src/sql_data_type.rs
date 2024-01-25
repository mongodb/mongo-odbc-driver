use num_derive::FromPrimitive;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(i16)]
pub enum SqlDataType {
    SQL_UNKNOWN_TYPE = 0,
    SQL_CHAR = 1,
    SQL_NUMERIC = 2,
    SQL_DECIMAL = 3,
    SQL_INTEGER = 4,
    SQL_SMALLINT = 5,
    SQL_FLOAT = 6,
    SQL_REAL = 7,
    SQL_DOUBLE = 8,
    SQL_DATETIME = 9,
    SQL_VARCHAR = 12,
    SQL_TYPE_DATE = 91,
    SQL_TYPE_TIME = 92,
    SQL_TYPE_TIMESTAMP = 93,
    SQL_INTERVAL = 10,
    SQL_TIMESTAMP = 11,
    SQL_LONGVARCHAR = -1,
    SQL_BINARY = -2,
    SQL_VARBINARY = -3,
    SQL_LONGVARBINARY = -4,
    SQL_BIGINT = -5,
    SQL_TINYINT = -6,
    SQL_BIT = -7,
    SQL_WCHAR = -8,
    SQL_WVARCHAR = -9,
    SQL_WLONGVARCHAR = -10,
    SQL_GUID = -11,

    // additional spec types: https://learn.microsoft.com/en-us/sql/odbc/reference/appendixes/sql-data-types?view=sql-server-ver16
    SQL_INTERVAL_YEAR = 101,
    SQL_INTERVAL_MONTH = 102,
    SQL_INTERVAL_DAY = 103,
    SQL_INTERVAL_HOUR = 104,
    SQL_INTERVAL_MINUTE = 105,
    SQL_INTERVAL_SECOND = 106,
    SQL_INTERVAL_YEAR_TO_MONTH = 107,
    SQL_INTERVAL_DAY_TO_HOUR = 108,
    SQL_INTERVAL_DAY_TO_MINUTE = 109,
    SQL_INTERVAL_DAY_TO_SECOND = 110,
    SQL_INTERVAL_HOUR_TO_MINUTE = 111,
    SQL_INTERVAL_HOUR_TO_SECOND = 112,
    SQL_INTERVAL_MINUTE_TO_SECOND = 113,
}
