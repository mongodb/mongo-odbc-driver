use crate::{conn::MongoConnection, err::Result, json_schema, stmt::MongoStatement, Error};
use bson::{doc, Bson, Document, RawBson};
use mongodb::{options::AggregateOptions, sync::Cursor};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeSet, HashMap}, time::Duration};

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
                let db = client.client.database(current_db);

                // 1. Run the $sql to get the result set cursor.
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

                let cursor = db.aggregate(pipeline, options)?;

                // 2. Run the sqlGetResultSchema command to get the result set
                // metadata. Sort the column metadata alphabetically (by column
                // name).
                let get_schema_cmd =
                    doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};
                let get_result_schema_response: SqlGetResultSchemaResponse =
                    bson::from_document(db.run_command(get_schema_cmd, None)?)
                        .map_err(Error::BsonError)?;

                // TODO:
                //   - build metadata vec out of md_result
                //   - md_result.schema.jsonSchema contains the json schema of the result set
                //   1. simplify json schema such that bsonType is a single field and polymorphic types are properly represented as anyOfs
                //   2. create metadata from simplified schema
                //      a. sort datasources alphabetically
                //      b. process each datasource:
                //         - sort field alphabetically
                //         - extract/calculate column info for each field

                Ok(MongoQuery {
                    resultset_cursor: cursor,
                    resultset_metadata: vec![],
                })
            }
        }
    }

    // Return the number of fields/columns in the resultset
    fn _get_col_count(&self) -> u32 {
        self.resultset_metadata.len() as u32
    }

    // Get the metadata for the column with the given index.
    fn _get_col_metadata(&self, col_index: u16) -> Result<&MongoColMetadata> {
        self.resultset_metadata
            .get(col_index as usize)
            .map_or(Err(Error::ColIndexOutOfBounds(col_index)), |md| Ok(md))
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        self.resultset_cursor.advance().map_err(Error::MongoError)
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let md = self._get_col_metadata(col_index)?;
        let datasource = self
            .resultset_cursor
            .deserialize_current()?
            .get_document(md.table_name.clone())
            .map_err(Error::ValueAccess)?;
        Ok(datasource.get(md.col_name.clone())) // TODO: why is this an issue and how can we address it?
    }
}

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAttribute or SQLDescribeCol and when converting the data to the targeted C type.
#[derive(Clone, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct SqlGetResultSchemaResponse {
    pub ok: i32,
    pub schema: VersionedJsonSchema,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct VersionedJsonSchema {
    pub version: i32,
    pub json_schema: json_schema::Schema,
}

pub struct SimplifiedJsonSchema {
    pub bson_type: String,
    pub properties: HashMap<String, Box<SimplifiedJsonSchema>>,
    pub any_of: Vec<Box<SimplifiedJsonSchema>>, // TODO: make set
    pub required: BTreeSet<String>,
    pub items: Box<SimplifiedJsonSchema>,
    pub additional_properties: bool,
}
