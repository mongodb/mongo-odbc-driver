use crate::conn::MongoConnection;
use crate::stmt::MongoStatement;
use bson::{Array, Bson, Document};
use mongodb::sync::Cursor;
use std::error::Error;

#[derive(Debug)]
pub struct MongoCollections {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The current row the cursor points to.
    current_row: Box<Array>,
}

// Statement related to a SQLColumns call.
impl MongoCollections {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_tables(
        client: &MongoConnection,
        query_timeout: Option<i32>,
        db_name_filter: &str,
        collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool, Box<dyn Error>> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<&Bson>, Box<dyn Error>> {
        unimplemented!()
    }
}
