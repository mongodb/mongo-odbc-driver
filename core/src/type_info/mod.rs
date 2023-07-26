use crate::definitions::SqlDataType;

mod simple_bson_type_info;
mod standard_bson_type_info;
pub mod mongo_type_info;

pub use simple_bson_type_info::SimpleBsonTypeInfo;
pub use standard_bson_type_info::StandardBsonTypeInfo;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TypeMode {
    Standard,
    Simple,
}

pub const SQL_SEARCHABLE: i32 = 3;
const SQL_PRED_BASIC: i32 = 2;
const SQL_PRED_NONE: i32 = 0;

#[derive(PartialEq, Debug, Clone)]
pub enum BsonTypeInfo {
    Standard(StandardBsonTypeInfo),
    Simple(SimpleBsonTypeInfo),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeInfoFields {
    pub type_name: &'static str,
    pub sql_type: SqlDataType,
    pub non_concise_type: SqlDataType,
    pub searchable: i32,
    pub is_case_sensitive: bool,
    pub fixed_prec_scale: bool,
    pub scale: Option<u16>,
    pub precision: Option<u16>,
    pub octet_length: Option<u16>,
    pub fixed_bytes_length: Option<u16>,
    pub literal_prefix: Option<&'static str>,
    pub literal_suffix: Option<&'static str>,
    pub sql_code: Option<i32>,
    pub is_auto_unique_value: Option<bool>,
    pub is_unsigned: Option<bool>,
    pub num_prec_radix: Option<u16>,
}
