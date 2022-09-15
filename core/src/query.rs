use crate::{conn::MongoConnection, err::Result, stmt::MongoStatement, Error};
use bson::{doc, Bson, Document};
use mongodb::{options::AggregateOptions, sync::Cursor};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The result set metadata.
    resultset_metadata: Vec<MongoColMetadata>,
}

impl MongoQuery {
    // Create a new MongoQuery on the connection's current database. Execute a
    // $sql aggregation with the given query and initialize the result set
    // cursor. If there is a timeout, the query must finish before the timeout
    // or an error is returned.
    pub fn execute(
        client: &MongoConnection,
        query_timeout: Option<i32>,
        query: &str,
    ) -> Result<Self> {
        match &client.current_db {
            None => Err(Error::NoDatabase),
            Some(current_db) => {
                let pipeline = vec![doc! {"$sql": {
                    "dialect": "mongosql",
                    "format": "odbc",
                    "statement": query,
                }}];

                let options = query_timeout.map(|i| {
                    AggregateOptions::builder()
                        .max_time(Duration::from_millis(i as u64)) // TODO: should timeout come from client if present?
                        .build()
                });

                let c = client
                    .client
                    .database(current_db)
                    .aggregate(pipeline, options)?;

                // TODO: run sql result schema command and store resultset_metadata

                Ok(MongoQuery {
                    resultset_cursor: c,
                    resultset_metadata: vec![],
                })
            }
        }
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
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        unimplemented!()
    }
}

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAttribute or SQLDescribeCol and when converting the data to the targeted C type.
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
