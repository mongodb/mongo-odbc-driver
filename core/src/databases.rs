use crate::conn::MongoConnection;
use crate::stmt::MongoStatement;
use bson::Bson;
use std::error::Error;

#[derive(Debug)]
pub struct MongoDatabases {
    // The list of all the databases
    databases_names: Vec<String>,
    // The current database index.
    current_row_index: u32,
}

// Statement for SQLTables(SQL_ALL_CATALOGS, "","").
impl MongoDatabases {
    // Create a new MongoStatement to list all the valid catalogs.
    // Correspond to SQLTables(SQL_ALL_CATALOGS, "","").
    // All columns except the TABLE_CAT column contain NULLs.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_all_catalogs(client: &MongoConnection, query_timeout: Option<i32>) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoDatabases {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool, Box<dyn Error>> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<&Bson>, Box<dyn Error>> {
        unimplemented!()
    }
}
