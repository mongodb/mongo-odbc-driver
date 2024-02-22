use num_derive::FromPrimitive;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(i16)]
pub enum Nullability {
    SQL_NO_NULLS = 0,
    SQL_NULLABLE = 1,
    SQL_NULLABLE_UNKNOWN = 2,
}
