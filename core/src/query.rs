use crate::{conn::MongoConnection, err::Result, json_schema, stmt::MongoStatement, Error, Schema};
use bson::{doc, Bson, Document, RawBson};
use itertools::Itertools;
use mongodb::{options::AggregateOptions, sync::Cursor};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

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

                // 1. Run the $sql aggregation to get the result set cursor.
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
                // metadata. Sort the column metadata alphabetically by column
                // name.
                let get_result_schema_cmd =
                    doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};

                let get_result_schema_response: SqlGetResultSchemaResponse =
                    bson::from_document(db.run_command(get_result_schema_cmd, None)?)
                        .map_err(Error::BsonDeserialization)?;

                let metadata = MongoQuery::process_metadata(get_result_schema_response);

                Ok(MongoQuery {
                    resultset_cursor: cursor,
                    resultset_metadata: metadata,
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

    fn process_metadata(
        get_result_schema_response: SqlGetResultSchemaResponse,
    ) -> Vec<MongoColMetadata> {
        let result_set_schema: SimplifiedJsonSchema =
            get_result_schema_response.schema.json_schema.into();

        // TODO: should we assert that result_set_schema is a Document schema since it must be?

        let x = result_set_schema
            .properties
            .into_iter()
            .sorted_by_key(|x| x.0);

        let datasources = result_set_schema.properties.keys().sorted();

        for ds in datasources {
            let datasource_schema = result_set_schema.properties.get(ds).unwrap();

            // TODO: should we assert that each datasource_schema is a Document schema sicne it must be?

            let fields = datasource_schema.properties.keys().sorted();

            for field in fields {
                // now we can process each field and add it to the metadata vec
            }
        }

        // TODO:
        //   2. create metadata from simplified schema
        //      a. sort datasources alphabetically
        //      b. process each datasource:
        //         - sort field alphabetically
        //         - extract/calculate column info for each field
        todo!()
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
            .get_document(&md.table_name)
            .map_err(Error::ValueAccess)?;
        Ok(datasource.get(&md.col_name))
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
    pub any_of: Vec<Box<SimplifiedJsonSchema>>,
    pub required: BTreeSet<String>,
    pub items: Box<SimplifiedJsonSchema>,
    pub additional_properties: bool,
}

// Converts a deserialized json_schema::Schema into a SimplifiedJsonSchema. The
// SimplifiedJsonSchema instance is semantically equivalent to the base schema,
// but bson_type has to be a single type otherwise the types will get pushed
// down in the any_of list.
impl From<json_schema::Schema> for SimplifiedJsonSchema {
    fn from(_: Schema) -> Self {
        todo!()
    }
}
