#![allow(dead_code)]
mod bson_type_info;
use bson_type_info::BsonTypeInfo;
mod collections;
pub use collections::MongoCollections;
mod conn;
pub use conn::MongoConnection;
mod databases;
pub use databases::MongoDatabases;
mod err;
pub use err::{Error, Result};
mod fields;
pub use fields::MongoFields;
mod col_metadata;
mod json_schema;
pub use col_metadata::MongoColMetadata;
mod query;
pub use query::MongoQuery;
mod stmt;
pub use stmt::MongoStatement;

#[cfg(test)]
mod unit {
    #[test]
    fn it_works() {
        // no-op
    }
}
