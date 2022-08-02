#![allow(dead_code)]
mod collections;
pub use collections::MongoCollections;
mod conn;
pub use crate::conn::MongoConnection;
mod databases;
pub use databases::MongoDatabases;
mod err;
pub use err::{Error, Result};
mod fields;
pub use fields::MongoFields;
mod query;
pub use query::{MongoColMetadata, MongoQuery};
mod stmt;
pub use stmt::MongoStatement;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // no-op
    }
}
