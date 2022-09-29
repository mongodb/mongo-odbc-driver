use crate::conn::MongoConnection;
use crate::err::Result;
use crate::stmt::MongoStatement;
use bson::Bson;
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
}

// Statement related to a SQLColumns call.
impl MongoCollections {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_tables(
        _client: &MongoConnection,
        _query_timeout: Option<i32>,
        _db_name_filter: &str,
        _collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next CollectionSpecification.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the given colIndex on the current CollectionSpecification.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> current_coll_list.0
        // 2 -> current_coll.name
        // 3 -> current_coll.CollectionType
        unimplemented!()
    }

    // Get the number of columns in the result set for this MongoCollections Statement.
    fn num_result_columns(&self) -> u16 {
        3
    }
}
