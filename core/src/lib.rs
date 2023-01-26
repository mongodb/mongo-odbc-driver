#![allow(dead_code)]
mod bson_type_info;
use bson_type_info::BsonTypeInfo;
mod collections;
pub use collections::MongoCollections;
mod conn;
pub use conn::MongoConnection;
mod databases;
pub use databases::MongoDatabases;
mod table_types;
pub use table_types::MongoTableTypes;
mod err;
pub use err::{Error, Result};
mod fields;
pub use fields::MongoFields;
pub mod col_metadata;
pub mod json_schema;
pub use col_metadata::MongoColMetadata;
mod query;
pub use query::MongoQuery;
pub mod mock_query;
mod stmt;
pub use stmt::MongoStatement;
pub mod odbc_uri;
mod primary_keys;
pub mod util;
pub use primary_keys::MongoPrimaryKeys;
mod foreign_keys;
pub use foreign_keys::MongoForeignKeys;

#[cfg(target_os = "macos")]
pub type WChar = u32;

#[cfg(not(target_os = "macos"))]
pub type WChar = u16;

#[cfg(target_os = "macos")]
pub fn from_wchar_vec_lossy(v: Vec<u32>) -> String {
    widestring::decode_utf32_lossy(v).collect::<String>()
}

#[cfg(target_os = "macos")]
pub fn from_wchar_ref_lossy(v: &[u32]) -> String {
    widestring::decode_utf32_lossy(v.iter().copied()).collect::<String>()
}

#[cfg(target_os = "macos")]
pub fn to_wchar_vec(s: &str) -> Vec<u16> {
    widestring::encode_utf32(s.chars()).collect::<Vec<_>>()
}

#[cfg(not(target_os = "macos"))]
pub fn from_wchar_vec_lossy(v: Vec<u16>) -> String {
    widestring::decode_utf16_lossy(v).collect::<String>()
}

#[cfg(not(target_os = "macos"))]
pub fn from_wchar_ref_lossy(v: &[u16]) -> String {
    widestring::decode_utf16_lossy(v.iter().copied()).collect::<String>()
}

#[cfg(not(target_os = "macos"))]
pub fn to_wchar_vec(s: &str) -> Vec<u16> {
    widestring::encode_utf16(s.chars()).collect::<Vec<_>>()
}
