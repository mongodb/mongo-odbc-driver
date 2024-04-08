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
pub const SQL_NTS: isize = -3;
pub const SQL_NTSL: isize = -3;

/// Maximum message length
pub const SQL_MAX_MESSAGE_LENGTH: SmallInt = 512;
pub const SQL_SQLSTATE_SIZE: usize = 5;

/// SQL Free Statement options
#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum FreeStmtOption {
    /// Closes the cursor associated with StatementHandle (if one was defined) and discards all
    /// pending results. The application can reopen this cursor later by executing a SELECT
    /// statement again with the same or different parameter values. If no cursor is open, this
    /// option has no effect for the application. `SQLCloseCursor` can also be called to close a
    /// cursor.
    SQL_CLOSE = 0,
    // SQL_DROP = 1, is deprecated in favour of SQLFreeHandle
    /// Sets the `SQL_DESC_COUNT` field of the ARD to 0, releasing all column buffers bound by
    /// `SQLBindCol` for the given StatementHandle. This does not unbind the bookmark column; to do
    /// that, the `SQL_DESC_DATA_PTR` field of the ARD for the bookmark column is set to NULL.
    /// Notice that if this operation is performed on an explicitly allocated descriptor that is
    /// shared by more than one statement, the operation will affect the bindings of all statements
    /// that share the descriptor.
    SQL_UNBIND = 2,
    /// Sets the `SQL_DESC_COUNT` field of the APD to 0, releasing all parameter buffers set by
    /// `SQLBindParameter` for the given StatementHandle. If this operation is performed on an
    /// explicitly allocated descriptor that is shared by more than one statement, this operation
    /// will affect the bindings of all the statements that share the descriptor.
    SQL_RESET_PARAMS = 3,
}

/// Represented in C headers as SQLSMALLINT
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i16)]
pub enum HandleType {
    SQL_HANDLE_ENV = 1,
    SQL_HANDLE_DBC = 2,
    SQL_HANDLE_STMT = 3,
    SQL_HANDLE_DESC = 4,
}

/// Options for `SQLDriverConnect`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(u16)]
pub enum DriverConnectOption {
    SQL_DRIVER_NO_PROMPT = 0,
    SQL_DRIVER_COMPLETE = 1,
    SQL_DRIVER_PROMPT = 2,
    SQL_DRIVER_COMPLETE_REQUIRED = 3,
}

// Attribute for string lengths
pub const SQL_IS_POINTER: i32 = -4;
pub const SQL_IS_UINTEGER: i32 = -5;
pub const SQL_IS_INTEGER: i32 = -6;
pub const SQL_IS_USMALLINT: i32 = -7;
pub const SQL_IS_SMALLINT: i32 = -8;

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

/// `DiagIdentifier` for `SQLGetDiagField`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum HeaderDiagnosticIdentifier {
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[repr(i32)]
pub enum AsyncConnectionBehavior {
    SQL_ASYNC_DBC_ENABLE_ON = 1,
    #[default]
    SQL_ASYNC_DBC_ENABLE_OFF = 0,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum DynamicDiagnosticIdentifier {
    SQL_DIAG_ALTER_DOMAIN = 3,
    SQL_DIAG_ALTER_TABLE = 4,
    SQL_DIAG_CALL = 7,
    SQL_DIAG_CREATE_ASSERTION = 6,
    SQL_DIAG_CREATE_CHARACTER_SET = 8,
    SQL_DIAG_CREATE_COLLATION = 10,
    SQL_DIAG_CREATE_DOMAIN = 23,
    SQL_DIAG_CREATE_INDEX = -1,
    SQL_DIAG_CREATE_SCHEMA = 64,
    SQL_DIAG_CREATE_TABLE = 77,
    SQL_DIAG_CREATE_TRANSLATION = 79,
    SQL_DIAG_CREATE_VIEW = 84,
    SQL_DIAG_DELETE_WHERE = 19,
    SQL_DIAG_DROP_ASSERTION = 24,
    SQL_DIAG_DROP_CHARACTER_SET = 25,
    SQL_DIAG_DROP_COLLATION = 26,
    SQL_DIAG_DROP_DOMAIN = 27,
    SQL_DIAG_DROP_INDEX = -2,
    SQL_DIAG_DROP_SCHEMA = 31,
    SQL_DIAG_DROP_TABLE = 32,
    SQL_DIAG_DROP_TRANSLATION = 33,
    SQL_DIAG_DROP_VIEW = 36,
    SQL_DIAG_DYNAMIC_DELETE_CURSOR = 38,
    SQL_DIAG_DYNAMIC_UPDATE_CURSOR = 81,
    SQL_DIAG_GRANT = 48,
    SQL_DIAG_INSERT = 50,
    SQL_DIAG_REVOKE = 59,
    SQL_DIAG_SELECT_CURSOR = 85,
    SQL_DIAG_UNKNOWN_STATEMENT = 0,
    SQL_DIAG_UPDATE_WHERE = 82,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(i16)]
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

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(i32)]
pub enum SqlBool {
    SQL_FALSE = 0,
    SQL_TRUE = 1,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(usize)]
pub enum CursorScrollable {
    SQL_NONSCROLLABLE = 0,
    SQL_SCROLLABLE = 1,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(usize)]
pub enum CursorSensitivity {
    SQL_UNSPECIFIED = 0,
    SQL_INSENSITIVE = 1,
    SQL_SENSITIVE = 2,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum AsyncEnable {
    SQL_ASYNC_ENABLE_OFF = 0,
    SQL_ASYNC_ENABLE_ON = 1,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(usize)]
pub enum Concurrency {
    SQL_CONCUR_READ_ONLY = 1,
    SQL_CONCUR_LOCK = 2,
    SQL_CONCUR_ROWVER = 3,
    SQL_CONCUR_VALUES = 4,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u32)]
pub enum CursorType {
    SQL_CURSOR_FORWARD_ONLY = 0,
    SQL_CURSOR_KEYSET_DRIVEN = 1,
    SQL_CURSOR_DYNAMIC = 2,
    SQL_CURSOR_STATIC = 3,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u32)]
pub enum NoScan {
    SQL_NOSCAN_OFF = 0,
    SQL_NOSCAN_ON = 1,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum BindType {
    SQL_BIND_BY_COLUMN = 0,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u32)]
pub enum RetrieveData {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum SimulateCursor {
    NonUnique = 0,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u32)]
pub enum UseBookmarks {
    Off = 0,
    Variable = 2,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, FromPrimitive)]
#[repr(usize)]
pub enum SqlCode {
    SQL_CODE_DATE = 1,
    SQL_CODE_TIME = 2,
    SQL_CODE_TIMESTAMP = 3,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum RowStatus {
    SQL_ROW_SUCCESS = 0,
    SQL_ROW_NOROW = 3,
    SQL_ROW_ERROR = 5,
    SQL_ROW_SUCCESS_WITH_INFO = 6,
}
