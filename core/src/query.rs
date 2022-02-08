use crate::conn::MongoConnection;
use crate::err::Result;
use crate::stmt::MongoStatement;
use bson::{Bson, Document};
use mongodb::sync::Cursor;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The result set metadata.
    resultset_metadata: Vec<MongoColMetadata>,
}

impl MongoQuery {
    // Create a new MongoStatement with StmtKind::Query on the connection currentDB.
    // Executes a $sql aggregation with the given query and initialize the Resultset cursor.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned
    pub fn execute(
        _client: &MongoConnection,
        _query_timeout: Option<i32>,
        _query: &str,
    ) -> Result<Self> {
        unimplemented!()
    }

    // Return the number of fields/columns in the resultset
    fn _get_col_count(&self) -> u32 {
        unimplemented!()
    }

    // Get the metadata for the column with the given index.
    fn _get_col_metadata(&self, _col_index: u16) -> Result<MongoColMetadata> {
        unimplemented!()
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<&Bson>> {
        unimplemented!()
    }
}

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAtrribute or SQLDescibeCol and when converting the data to the targeted C type.
#[derive(Debug)]
pub struct MongoColMetadata {
    pub base_col_name: String,
    pub base_table_name: String,
    pub catalog_name: String,
    pub col_count: u16,
    pub display_size: u64,
    pub fixed_prec_scale: bool,
    pub label: String,
    pub length: u128,
    pub col_name: String,
    pub is_nullable: bool,
    pub octet_length: u128,
    pub precision: u16,
    pub scale: u16,
    pub is_searchable: bool,
    pub table_name: String,
    // BSON type name
    pub type_name: String,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}
