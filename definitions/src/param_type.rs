use num_derive::FromPrimitive;

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
