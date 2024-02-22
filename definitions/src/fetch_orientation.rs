use num_derive::FromPrimitive;

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum FetchOrientation {
    SQL_FETCH_NEXT = 1,
    SQL_FETCH_FIRST = 2,
    SQL_FETCH_LAST = 3,
    SQL_FETCH_PRIOR = 4,
    SQL_FETCH_ABSOLUTE = 5,
    SQL_FETCH_RELATIVE = 6,
    SQL_FETCH_FIRST_USER = 31,
    SQL_FETCH_FIRST_SYSTEM = 32,
}
