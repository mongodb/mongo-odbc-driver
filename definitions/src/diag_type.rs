use num_derive::FromPrimitive;

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
