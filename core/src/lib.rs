mod collections;
pub use collections::MongoCollections;
mod conn;
pub use conn::MongoConnection;
mod databases;
pub use databases::MongoDatabases;
mod err;
mod fields;
pub use fields::MongoFields;
mod query;
pub use query::{MongoColMetadata, MongoQuery};
mod stmt;
pub use err::{Error, Result};
pub use stmt::MongoStatement;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // no-op
    }
}
