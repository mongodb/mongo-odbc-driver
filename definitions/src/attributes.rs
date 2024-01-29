use crate::Pointer;
use num_derive::FromPrimitive;

/// Governs behaviour of EnvironmentAttribute
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(i32)]
pub enum EnvironmentAttribute {
    SQL_ATTR_ODBC_VERSION = 200,
    SQL_ATTR_CONNECTION_POOLING = 201,
    SQL_ATTR_CP_MATCH = 202,
    SQL_ATTR_OUTPUT_NTS = 10001,
    // this is an iODBC specific attribute that the DM uses to check which encoding we're using.
    // We currently have the cstr/utf32 feature to handle encoding issues with iODBC, however if
    // in the future if we can use utf16 with iODBC, this will be useful.
    SQL_ATTR_DRIVER_UNICODE_TYPE = 1065,
}

/// ODBC verions
///
/// Possible values for `OdbcVersion` attribute set with `SQLSetEnvAttr` to declare ODBC version
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(u32)]
pub enum AttrOdbcVersion {
    // Not supported by this crate
    SQL_OV_ODBC2 = 2,
    SQL_OV_ODBC3 = 3,
    #[cfg(feature = "odbc_version_3_80")]
    SQL_OV_ODBC3_80 = 380,
    #[cfg(feature = "odbc_version_4")]
    SQL_OV_ODBC4 = 400,
}

impl From<AttrOdbcVersion> for Pointer {
    fn from(source: AttrOdbcVersion) -> Pointer {
        source as i32 as Pointer
    }
}
/// Connection pool configuration
///
/// Possible values for `ConnectionPooling` attribute set with `SQLSetEnvAttr` to define which
/// pooling scheme will be used.
///
/// See: <https://docs.microsoft.com/en-us/sql/odbc/reference/syntax/sqlsetenvattr-function>
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum AttrConnectionPooling {
    /// Connection pooling is turned off. This is the default.
    SQL_CP_OFF = 0,
    /// A single connection pool is supported for each driver. Every connection in a pool is
    /// associated with one driver.
    SQL_CP_ONE_PER_DRIVER = 1,
    /// A single connection pool is supported for each environment. Every connection in a pool is
    /// associated with one environment.
    SQL_CP_ONE_PER_HENV = 2,
    /// Use the connection-pool awareness feature of the driver, if it is available. If the driver
    /// does not support connection-pool awareness, `DriverAware` is ignored and `OnePerHenv` is
    /// used.
    SQL_CP_DRIVER_AWARE = 3,
}

/// Connection pool default configuration
impl Default for AttrConnectionPooling {
    fn default() -> Self {
        AttrConnectionPooling::SQL_CP_OFF
    }
}

impl From<AttrConnectionPooling> for Pointer {
    fn from(source: AttrConnectionPooling) -> Pointer {
        source as u32 as Pointer
    }
}

/// Determines how a connection is chosen from a connection pool.
///
/// Possible values for `CpMatch` attribute set with [`crate::SQLSetEnvAttr`] to define which connection
/// attributes must match for a connection returned from the pool
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum AttrCpMatch {
    /// Only connections that exactly match the connection options in the call and the connection
    /// attributes set by the application are reused. This is the default.
    SQL_CP_STRICT_MATCH = 0,
    /// Connections with matching connection string keywords can be used. Keywords must match, but
    /// not all connection attributes must match.
    SQL_CP_RELAXED_MATCH = 1,
}

/// Default matching for connections returned from the pool
impl Default for AttrCpMatch {
    fn default() -> Self {
        AttrCpMatch::SQL_CP_STRICT_MATCH
    }
}

impl From<AttrCpMatch> for Pointer {
    fn from(source: AttrCpMatch) -> Pointer {
        source as u32 as Pointer
    }
}

const SQL_ATTR_ASYNC_ENABLE: i32 = 4;
const SQL_ATTR_METADATA_ID: i32 = 10014;

/// Statement attributes are characteristics of the statement. For example, whether to use bookmarks
/// and what kind of cursor to use with the statement's result set are statement attributes.
///
/// Statement attributes are set with `SQLSetStmtAttr` and their current settings retrieved with
/// `SQLGetStmtAttr`. There is no requirement that an application set any statement attributes; all
/// statement attributes have defaults, some of which are driver-specific.
/// When a statement attribute can be set depends on the attribute itself. The
/// `Concurrency`, `CursorType, `SimulateCursor`, and `UseBookmars` statement attributes must be set
/// before the statement is executed. The `AsyncEnable` and `NoScan` statement attributes can be set
/// at any time but are not applied until the statement is used again. `MaxLength`, `MaxRows`, and
/// `QueryTimeout` statement attributes can be set at any time, but it is driver-specific whether
/// they are applied before the statement is used again. The remaining statement attributes can be
/// set at any time.
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
    SQL_ATTR_ASYNC_ENABLE = SQL_ATTR_ASYNC_ENABLE,
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
    SQL_ATTR_METADATA_ID = SQL_ATTR_METADATA_ID,
}

/// Connection attributes for `SQLSetConnectAttr`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(i32)]
pub enum ConnectionAttribute {
    SQL_ATTR_ASYNC_ENABLE = SQL_ATTR_ASYNC_ENABLE,
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
    SQL_ATTR_METADATA_ID = SQL_ATTR_METADATA_ID,
}
