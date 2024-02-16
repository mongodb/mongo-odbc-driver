use num_derive::FromPrimitive;

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
}

#[cfg(windows)]
pub use CDataType::SQL_C_ULONG as UBigInt;
#[cfg(not(windows))]
pub use CDataType::SQL_C_ULONG as Bookmark;
