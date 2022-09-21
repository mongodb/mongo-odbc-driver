use crate::conn::MongoConnection;
use crate::err::Result;
use crate::stmt::MongoStatement;
use bson::Bson;

#[derive(Debug)]
pub struct MongoDatabases {
    // The list of all the databases
    database_names: Vec<String>,
    // The current database index.
    current_db_index: usize,
}

// Statement for SQLTables(SQL_ALL_CATALOGS, "","").
impl MongoDatabases {
    // Create a new MongoStatement to list all the valid databases.
    // Correspond to SQLTables(SQL_ALL_CATALOGS, "","").
    // All columns except the TABLE_CAT column contain NULLs.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_all_catalogs(
        mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
    ) -> Self {
        let database_names: Vec<String> = mongo_connection
            .client
            .list_database_names(None, None)
            .unwrap();
        MongoDatabases {
            database_names,
            current_db_index: 0,
        }
    }
}

impl MongoStatement for MongoDatabases {
    // Increment current_db_index.
    // Return true if current_db_index index is <= for databases_names.length.
    fn next(&mut self) -> Result<bool> {
        self.current_db_index += 1;
        Ok(self.current_db_index <= self.database_names.len())
    }

    // Get the BSON value for the value at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> databases_names[current_row_index]
        match col_index {
            1 => Ok(Some(Bson::String(
                self.database_names
                    .get(self.current_db_index - 1)
                    .unwrap()
                    .to_string(),
            ))),
            _ => Ok(Some(Bson::Null)),
            // SQL-1031: Add database listing edge case handling
            // Col_or_Param_Num was greater than the number of columns in the result set
            // Or value specified for the argument Col_or_Param_Num was 0,
            // and the SQL_ATTR_USE_BOOKMARKS statement attribute was set to SQL_UB_OFF
            // Throw error 07009
        }
    }
}
