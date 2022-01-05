use crate::conn::MongoConnection;
use crate::resultset::MongoResultSet;
use bson::Bson;
use std::error::Error;

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
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection (tables) names filters.
    pub fn list_columns(
        client: &MongoConnection,
        db_name_filter: &str,
        collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoResultSet for MongoFields {
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
