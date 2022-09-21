use crate::conn::MongoConnection;
use crate::err::Result;
use crate::stmt::MongoStatement;
use bson::{doc, Bson, Document};
use mongodb::results::CollectionSpecification;
use mongodb::sync::Cursor;

#[derive(Debug)]
struct CollectionsForDb {
    database_name: String,
    collection_list: Cursor<CollectionSpecification>,
}

#[derive(Debug)]
pub struct MongoCollections {
    // The current collection specification.
    current_collection: Option<CollectionSpecification>,
    // The cursor on the collections specification for the db.
    current_collection_list: Option<CollectionsForDb>,
    // List of CollectionsForDb for each db.
    collections_for_db_list: Option<Vec<CollectionsForDb>>,
}

// Statement related to a SQLColumns call.
impl MongoCollections {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_tables(
        mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
        db_name_filter: &str,
        collection_name_filter: &str,
    ) -> Self {
        let mut databases: Vec<CollectionsForDb> = vec![];

        let names = mongo_connection
            .client
            .list_database_names(to_name_regex(db_name_filter), None)
            .unwrap();
        for name in names {
            let db = mongo_connection.client.database(name.as_str());
            let collections = db
                .list_collections(to_name_regex(collection_name_filter), None)
                .unwrap();
            databases.push(CollectionsForDb {
                database_name: name,
                collection_list: collections,
            });
        }
        MongoCollections {
            current_collection: None,
            current_collection_list: None,
            collections_for_db_list: Some(databases),
        }
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next CollectionSpecification.
    // When cursor is exhausted move to next database in list
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        if self.current_collection_list.is_none() {
            if self.collections_for_db_list.as_ref().unwrap().is_empty() {
                return Ok(false);
            }
            self.current_collection_list =
                Some(self.collections_for_db_list.as_mut().unwrap().remove(0));
        }
        loop {
            if self
                .current_collection_list
                .as_mut()
                .unwrap()
                .collection_list
                .advance()?
            {
                // Cursor advance succeeded, update current CollectionSpecification
                self.current_collection = Some(
                    self.current_collection_list
                        .as_ref()
                        .unwrap()
                        .collection_list
                        .deserialize_current()
                        .unwrap(),
                );
                return Ok(true);
            }
            if self.collections_for_db_list.as_ref().unwrap().is_empty() {
                return Ok(false);
            }
            // Current cursor exhausted, try next database in list
            self.current_collection_list =
                Some(self.collections_for_db_list.as_mut().unwrap().remove(0));
        }
    }

    // Get the BSON value for the given colIndex on the current CollectionSpecification.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> current_collection_list.database_name
        // 2-> Schema name; NULL as it is not applicable
        // 3 -> current_collection.name
        // 4 -> current_collection.collection_type
        match col_index {
            1 => Ok(Some(Bson::String(
                self.current_collection_list
                    .as_ref()
                    .unwrap()
                    .database_name
                    .clone(),
            ))),
            2 => Ok(Some(Bson::Null)),
            3 => Ok(Some(Bson::String(
                self.current_collection.as_ref().unwrap().name.clone(),
            ))),
            4 => Ok(Some(Bson::String(format!(
                "{:?}",
                self.current_collection.as_ref().unwrap().collection_type
            )))),
            _ => Ok(Some(Bson::Null)),
        }
    }
}

// Replaces SQL wildcard characters with associated regex
// Returns a doc applying filter to name
fn to_name_regex(filter: &str) -> Document {
    let regex_filter = filter.replace('%', ".*").replace('_', ".");
    doc! { "name": { "$regex": regex_filter } }
}
