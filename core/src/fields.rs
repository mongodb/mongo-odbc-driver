use crate::conn::MongoConnection;
use bson::Bson;
use std::error::Error;
use stmt::MongoStatement;

#[derive(Debug)]
pub struct MongoFields {
    // The list of all the databases
    databases_names: Vec<String>,

    // The current database index.
    current_db_index: u32,
    // The current collection index.
    current_collection_index: u32,
}

// Statement related to a SQLTables call.
// The Resultset columns are hard-coded and follow the ODBC resultset for SQLColumns :
// TABLE_CAT, TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE.
impl MongoFields {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_columns(
        client: &MongoConnection,
        query_timeout: Option<i32>,
        db_name_filter: &str,
        collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoFields {
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
