//! ODBC types those representation is compatible with the ODBC C API.
//!
//! This layer has not been created using automatic code generation. It is incomplete, i.e. it does
//! not contain every symbol or constant defined in the ODBC C headers. Symbols which are
//! deprecated since ODBC 3 have been left out intentionally. While some extra type safety has been
//! added by grouping some of C's `#define` constants into `enum`-types it mostly offers the same
//! power (all) and safety guarantess(none) as the wrapped C-API.
//! ODBC 4.0 is still under development by Microsoft, so these symbols are deactivated by default
//! in the cargo.toml

pub use self::{
    attributes::*, bulk_operation::*, c_data_type::*, desc::*, diag_type::*, fetch_orientation::*,
    functions::*, indicator::*, info_type::*, interval::*, nullability::*, param_type::*,
    sql_data_type::*, sqlreturn::*,
};
use cstr::WideChar;
use num_derive::FromPrimitive;
use std::os::raw::{c_int, c_void};

mod attributes;
mod bulk_operation;
mod c_data_type;
mod desc;
mod diag_type;
mod fetch_orientation;
mod functions;
mod indicator;
mod info_type;
mod interval;
mod nullability;
mod param_type;
mod sql_data_type;
mod sqlreturn;

#[cfg(feature = "iodbc")]
pub const USING_IODBC: bool = true;

#[cfg(not(feature = "iodbc"))]
pub const USING_IODBC: bool = false;

//These types can never be instantiated in Rust code.
pub enum Obj {}

pub enum Env {}

pub enum Dbc {}

pub enum Stmt {}

pub enum Description {}

pub type Handle = *mut Obj;
pub type HEnv = *mut Env;
pub type HDesc = *mut Description;

/// The connection handle references storage of all information about the connection to the data
/// source, including status, transaction state, and error information.
pub type HDbc = *mut Dbc;
pub type HStmt = *mut Stmt;

pub type SmallInt = i16;
pub type USmallInt = u16;
pub type Integer = i32;
pub type UInteger = u32;
pub type Pointer = *mut c_void;
pub type Char = u8;
pub type SChar = i8;
pub type WChar = WideChar;

pub type Len = isize;
pub type ULen = usize;

pub type HWnd = Pointer;

pub type RetCode = i16;

// Diag constants
pub const SQL_ROW_NUMBER_UNKNOWN: isize = -2;

// flags for null-terminated string
pub const NTS: isize = -3;
pub const NTSL: isize = -3;

/// Maximum message length
pub const MAX_MESSAGE_LENGTH: SmallInt = 512;
pub const SQLSTATE_SIZE: usize = 5;
pub const SQLSTATE_SIZEW: usize = 10;

/// SQL Free Statement options
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FreeStmtOption {
    /// Closes the cursor associated with StatementHandle (if one was defined) and discards all
    /// pending results. The application can reopen this cursor later by executing a SELECT
    /// statement again with the same or different parameter values. If no cursor is open, this
    /// option has no effect for the application. `SQLCloseCursor` can also be called to close a
    /// cursor.
    Close = 0,
    // SQL_DROP = 1, is deprecated in favour of SQLFreeHandle
    /// Sets the `SQL_DESC_COUNT` field of the ARD to 0, releasing all column buffers bound by
    /// `SQLBindCol` for the given StatementHandle. This does not unbind the bookmark column; to do
    /// that, the `SQL_DESC_DATA_PTR` field of the ARD for the bookmark column is set to NULL.
    /// Notice that if this operation is performed on an explicitly allocated descriptor that is
    /// shared by more than one statement, the operation will affect the bindings of all statements
    /// that share the descriptor.
    Unbind = 2,
    /// Sets the `SQL_DESC_COUNT` field of the APD to 0, releasing all parameter buffers set by
    /// `SQLBindParameter` for the given StatementHandle. If this operation is performed on an
    /// explicitly allocated descriptor that is shared by more than one statement, this operation
    /// will affect the bindings of all the statements that share the descriptor.
    ResetParams = 3,
}

/// Represented in C headers as SQLSMALLINT
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HandleType {
    Env = 1,
    Dbc = 2,
    Stmt = 3,
    Desc = 4,
}

/// Options for `SQLDriverConnect`
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DriverConnectOption {
    NoPrompt = 0,
    Complete = 1,
    Prompt = 2,
    CompleteRequired = 3,
}

// Attribute for string lengths

/// SQL_IS_POINTER
pub const IS_POINTER: i32 = -4;
/// SQL_IS_UINTEGER
pub const IS_UINTEGER: i32 = -5;
/// SQL_IS_INTEGER
pub const IS_INTEGER: i32 = -6;
/// SQL_IS_USMALLINT
pub const IS_USMALLINT: i32 = -7;
/// SQL_IS_SMALLINT
pub const IS_SMALLINT: i32 = -8;

/// SQL_YEAR_MONTH_STRUCT
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct YearMonth {
    pub year: UInteger,
    pub month: UInteger,
}

/// SQL_DAY_SECOND_STRUCT
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DaySecond {
    pub day: UInteger,
    pub hour: UInteger,
    pub minute: UInteger,
    pub second: UInteger,
    pub fraction: UInteger,
}

/// SQL_INTERVAL_UNION
#[repr(C)]
#[derive(Copy, Clone)]
pub union IntervalUnion {
    pub year_month: YearMonth,
    pub day_second: DaySecond,
}

/// SQL_INTERVAL_STRUCT
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IntervalStruct {
    pub interval_type: c_int,
    pub interval_sign: SmallInt,
    pub interval_value: IntervalUnion,
}

/// SQL_DATE_STRUCT
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Date {
    pub year: SmallInt,
    pub month: USmallInt,
    pub day: USmallInt,
}

/// SQL_TIME_STRUCT
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Time {
    pub hour: USmallInt,
    pub minute: USmallInt,
    pub second: USmallInt,
}

/// SQL_TIMESTAMP_STRUCT
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Timestamp {
    pub year: SmallInt,
    pub month: USmallInt,
    pub day: USmallInt,
    pub hour: USmallInt,
    pub minute: USmallInt,
    pub second: USmallInt,
    pub fraction: UInteger,
}

/// SQLGUID
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Guid {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

/// Connection attributes for `SQLSetConnectAttr`
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

/// `DiagIdentifier` for `SQLGetDiagField`
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HeaderDiagnosticIdentifier {
    /// SQL_DIAG_RETURNCODE
    ReturnCode = 1,
    /// SQL_DIAG_NUMBER
    Number = 2,
    /// SQL_DIAG_ROW_COUNT
    RowCount = 3,
    /// SQL_DIAG_SQLSTATE
    SqlState = 4,
    /// SQL_DIAG_NATIVE
    Native = 5,
    /// SQL_DIAG_MESSAGE_TEXT
    MessageText = 6,
    /// SQL_DIAG_DYNAMIC_FUNCTION
    DynamicFunction = 7,
    /// SQL_DIAG_CLASS_ORIGIN
    ClassOrigin = 8,
    /// SQL_DIAG_SUBCLASS_ORIGIN
    SubclassOrigin = 9,
    /// SQL_DIAG_CONNECTION_NAME
    ConnectionName = 10,
    /// SQL_DIAG_SERVER_NAME
    ServerName = 11,
    /// SQL_DIAG_DYNAMIC_FUNCTION_CODE
    DynamicFunctionCode = 12,
    /// SQL_DIAG_CURSOR_ROW_COUNT
    CursorRowCount = -1249,
    /// SQL_DIAG_ROW_NUMBER
    RowNumber = -1248,
    /// SQL_DIAG_COLUMN_NUMBER
    ColumnNumber = -1247,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum AsyncConnectionBehavior {
    /// SQL_ASYNC_DBC_ENABLE_ON
    On = 1,
    /// SQL_ASYNC_DBC_ENABLE_OFF = 0
    #[default]
    Off = 0,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DynamicDiagnosticIdentifier {
    /// SQL_DIAG_ALTER_DOMAIN
    AlterDomain = 3,
    /// SQL_DIAG_ALTER_TABLE,
    AlterTable = 4,
    /// SQL_DIAG_CALL
    Call = 7,
    /// SQL_DIAG_CREATE_ASSERTION
    CreateAssertion = 6,
    /// SQL_DIAG_CREATE_CHARACTER_SET
    CreateCharacterSet = 8,
    /// SQL_DIAG_CREATE_COLLATION,
    CreateCollation = 10,
    /// SQL_DIAG_CREATE_DOMAIN
    CreateDomain = 23,
    /// SQL_DIAG_CREATE_INDEX
    CreateIndex = -1,
    /// SQL_DIAG_CREATE_SCHEMA
    CreateSchema = 64,
    /// SQL_DIAG_CREATE_TABLE
    CreateTable = 77,
    /// SQL_DIAG_CREATE_TRANSLATION
    CreateTranslation = 79,
    /// SQL_DIAG_CREATE_VIEW
    CreateView = 84,
    /// SQL_DIAG_DELETE_WHERE
    DeleteWhere = 19,
    /// SQL_DIAG_DROP_ASSERTION
    DropAssertion = 24,
    /// SQL_DIAG_DROP_CHARACTER_SET
    DropCharacterSet = 25,
    /// SQL_DIAG_DROP_COLLATION
    DropCollation = 26,
    /// SQL_DIAG_DROP_DOMAIN
    DropDomain = 27,
    /// SQL_DIAG_DROP_INDEX
    DropIndex = -2,
    /// SQL_DIAG_DROP_SCHEMA
    DropSchema = 31,
    /// SQL_DIAG_DROP_TABLE
    DropTable = 32,
    /// SQL_DIAG_DROP_TRANSLATION
    DropTranslation = 33,
    /// SQL_DIAG_DROP_VIEW
    DropView = 36,
    /// SQL_DIAG_DYNAMIC_DELETE_CURSOR
    DynamicDeleteCursor = 38,
    /// SQL_DIAG_DYNAMIC_UPDATE_CURSOR
    DynamicUpdateCursor = 81,
    /// SQL_DIAG_GRANT
    Grant = 48,
    /// SQL_DIAG_INSERT
    Insert = 50,
    /// SQL_DIAG_REVOKE
    Revoke = 59,
    // SQL_DIAG_SELECT_CURSOR
    SelectCursor = 85,
    /// SQL_DIAG_UNKNOWN_STATEMENT = 0,
    UnknownStatement = 0,
    /// SQL_DIAG_UPDATE_WHERE = 82,
    UpdateWhere = 82,
}

#[allow(non_camel_case_types)]
#[repr(i16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum CompletionType {
    SQL_COMMIT = 0,
    SQL_ROLLBACK = 1,
}

pub const MAX_NUMERIC_LEN: usize = 16;
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Numeric {
    pub precision: Char,
    /// Number of decimal digits to the right of the decimal point.
    pub scale: SChar,
    /// 1 if positive, 0 if negative
    pub sign: Char,
    pub val: [Char; MAX_NUMERIC_LEN],
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum SqlBool {
    False = 0,
    True,
}

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
