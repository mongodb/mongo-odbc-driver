use crate::conn::MongoConnection;
use crate::error::Error;
use crate::stmt::MongoStatement;
use bson::Bson;

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
    pub fn list_all_catalogs(client: &MongoConnection) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoDatabases {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool, Error> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<&Bson>, Error> {
        unimplemented!()
    }
}
