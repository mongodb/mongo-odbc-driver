use crate::conn::MongoConnection;
use crate::err::Result;
use crate::stmt::MongoStatement;
use bson::{Bson, Document};
use mongodb::sync::Cursor;

#[derive(Debug)]
struct FieldsForCollection {
    database_name: String,
    collection_name: String,
    // Info retrieved via sqlgetschema
    // See https://docs.mongodb.com/datalake/reference/cli/sql/sqlgetschema/ for more details.
    schema: Cursor<Document>,
}

#[derive(Debug)]
pub struct MongoFields {
    // The current collection specification.
    current_field_list: Option<FieldsForCollection>,
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
        _client: &MongoConnection,
        _query_timeout: Option<i32>,
        _db_name_filter: &str,
        _collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoFields {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        unimplemented!()
    }
}
