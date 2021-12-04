use crate::conn::MongoConnection;
use crate::error::Error;
use crate::stmt::MongoStatement;
use bson::{Array, Bson, Document};
use mongodb::sync::Cursor;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The current row the cursor points to.
    current_row: Box<Array>,
    // The result set metadata.
    resultset_metadata: Vec<MongoColMetadata>,
}

impl MongoQuery {
    // Create a new MongoStatement with StmtKind::Query on the connection currentDB.
    // Executes a $sql aggregation with the given query and initialize the Resultset cursor.
    pub fn execute(client: &MongoConnection, query: &str) -> Result<Self, Error> {
        unimplemented!()
    }

    // Return the number of fields/columns in the resultset
    fn get_col_count(&self) -> u32 {
        unimplemented!()
    }

    // Get the metadata for the column with the given index.
    fn get_col_metadata(&self, col_index: u16) -> Result<MongoColMetadata, Error> {
        unimplemented!()
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool, Error> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<&Bson>, Error> {
        unimplemented!()
    }
}

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAtrribute or SQLDescibeCol and when converting the data to the targeted C type.
#[derive(Debug)]
pub struct MongoColMetadata {
    pub base_col_name: Box<str>,
    pub base_table_name: Box<str>,
    pub catalog_name: Box<str>,
    pub col_count: u16,
    pub display_size: u64,
    pub fixed_prec_scale: bool,
    pub label: Box<str>,
    pub length: u128,
    pub col_name: Box<str>,
    pub is_nullable: bool,
    pub octet_length: u128,
    pub precision: u16,
    pub scale: u16,
    pub is_searchable: bool,
    pub table_name: Box<str>,
    // BSON type name
    pub type_name: Box<str>,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}
// MongoDB doesn't have auto-incremented fields
// pub is_auto_unique: bool,

// This is always true
// pub is_case_sensitive: bool,

// Always null
//pub schema_name: str

// Only used for IRD
//pub unnamed: u16,

// Conversion from BSON to SQL will be done by the ODBC component
//pub type: SQLDataType,
//pub concise_type: SqlDataType,

// Can be inferred from the SQL type
//pub num_prec_radix: u16,
//pub literal_prefix: str,
//pub literal_suffix: str,

// N/A
//pub local_type_name: str,
