use crate::bson_type_info::BsonTypeInfo;
use crate::{
    conn::MongoConnection,
    err::Result,
    json_schema::{self, simplified::ObjectSchema, BsonTypeName},
    stmt::MongoStatement,
    Error,
};
use bson::{doc, Bson, Document};
use itertools::Itertools;
use mongodb::{options::AggregateOptions, sync::Cursor};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current deserialized "row".
    current: Option<Document>,
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
        let current_db = client.current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = client.client.database(current_db);

        // 1. Run the sqlGetResultSchema command to get the result set
        // metadata. Column metadata is sorted alphabetically by table
        // and column name.
        let get_result_schema_cmd =
            doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};

        let get_result_schema_response: SqlGetResultSchemaResponse =
            bson::from_document(db.run_command(get_result_schema_cmd, None)?)
                .map_err(Error::BsonDeserialization)?;

        let metadata = get_result_schema_response.process_metadata(current_db)?;

        // 2. Run the $sql aggregation to get the result set cursor.
        let pipeline = vec![doc! {"$sql": {
            "format": "odbc",
            "formatVersion": 1,
            "statement": query,
        }}];

        let options = query_timeout.map(|i| {
            AggregateOptions::builder()
                .max_time(Duration::from_millis(i as u64))
                .build()
        });

        let cursor = db.aggregate(pipeline, options)?;

        Ok(MongoQuery {
            resultset_cursor: cursor,
            resultset_metadata: metadata,
            current: None,
        })
    }

    // Return the number of fields/columns in the resultset
    fn _get_col_count(&self) -> u32 {
        self.resultset_metadata.len() as u32
    }

    // Get the metadata for the column with the given index.
    fn get_col_metadata(&self, col_index: u16) -> Result<&MongoColMetadata> {
        self.resultset_metadata
            .get(col_index as usize)
            .map_or(Err(Error::ColIndexOutOfBounds(col_index)), Ok)
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self) -> Result<bool> {
        let res = self.resultset_cursor.advance().map_err(Error::Mongo);
        self.current = Some(self.resultset_cursor.deserialize_current()?);
        res
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let current = self.current.as_ref().ok_or(Error::InvalidCursorState)?;
        let md = self.get_col_metadata(col_index)?;
        let datasource = current
            .get_document(&md.table_name)
            .map_err(Error::ValueAccess)?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    // Get the number of columns in the result set for this Query Statement.
    fn num_result_columns(&self) -> u16 {
        self.resultset_metadata.len() as u16
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
    pub display_size: Option<u16>,
    pub fixed_prec_scale: bool,
    pub label: String,
    pub length: Option<u16>,
    pub col_name: String,
    pub is_nullable: ColumnNullability,
    pub octet_length: Option<u16>,
    pub precision: Option<u16>,
    pub scale: Option<u16>,
    pub is_searchable: bool,
    pub table_name: String,
    // BSON type name
    pub type_name: String,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}

// Struct representing the response for a sqlGetResultSchema command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct SqlGetResultSchemaResponse {
    pub ok: i32,
    pub schema: VersionedJsonSchema,
}

// Auxiliary struct representing part of the response for a sqlGetResultSchema
// command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct VersionedJsonSchema {
    pub version: i32,
    pub json_schema: json_schema::Schema,
}

impl SqlGetResultSchemaResponse {
    /// Converts a sqlGetResultSchema command response into a list of column
    /// metadata. Ensures the top-level schema is an Object with properties,
    /// and ensures the same for each top-level property -- which correspond
    /// to datasources. The metadata is sorted alphabetically by datasource
    /// name and then by field name. As in, a result set with schema:
    ///
    ///   {
    ///     bsonType: "object",
    ///     properties: {
    ///       "foo": {
    ///         bsonType: "object",
    ///         properties: { "b": { bsonType: "int" }, "a": { bsonType: "string" } }
    ///       },
    ///       "bar": {
    ///         bsonType: "object",
    ///         properties: { "c": { bsonType: "int" } }
    ///       }
    ///   }
    ///
    /// produces a list of metadata with the order: "bar.c", "foo.a", "foo.b".
    fn process_metadata(&self, current_db: &str) -> Result<Vec<MongoColMetadata>> {
        let result_set_schema: json_schema::simplified::Schema =
            self.schema.json_schema.clone().try_into()?;
        let result_set_object_schema = result_set_schema.assert_datasource_schema()?;

        let sorted_datasource_object_schemas = result_set_object_schema
            .clone()
            // 1. Access result_set_schema.properties and sort alphabetically.
            //    This means we are sorting by datasource name.
            .properties
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            .map(|(datasource_name, datasource_schema)| {
                let obj_schema = datasource_schema.assert_datasource_schema()?;

                Ok((datasource_name, obj_schema.clone()))
            })
            .collect::<Result<Vec<(String, ObjectSchema)>>>()?;

        sorted_datasource_object_schemas
            .into_iter()
            // 2. Flat-map fields for each datasource, sorting fields alphabetically.
            .flat_map(|(datasource_name, datasource_schema)| {
                datasource_schema
                    .clone()
                    .properties
                    .into_iter()
                    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                    .map(move |(field_name, field_schema)| {
                        (
                            datasource_name.clone(),
                            datasource_schema.clone(),
                            field_name,
                            field_schema,
                        )
                    })
            })
            // 3. Map each field into a MongoColMetadata.
            .map(
                |(datasource_name, datasource_schema, field_name, field_schema)| {
                    let field_nullability =
                        datasource_schema.get_field_nullability(field_name.clone())?;

                    Ok(Self::create_column_metadata(
                        current_db,
                        datasource_name,
                        field_name,
                        field_schema,
                        field_nullability,
                    ))
                },
            )
            // 4. Collect as a Vec.
            .collect::<Result<Vec<MongoColMetadata>>>()
    }

    fn create_column_metadata(
        _current_db: &str,
        datasource_name: String,
        field_name: String,
        field_schema: json_schema::simplified::Schema,
        is_nullable: ColumnNullability,
    ) -> MongoColMetadata {
        let bson_type_info: BsonTypeInfo = field_schema.into();

        MongoColMetadata {
            // For base_col_name, base_table_name, and catalog_name, we do
            // not have this information in sqlGetResultSchema, so these will
            // always be empty string for now.
            base_col_name: "".to_string(),
            base_table_name: "".to_string(),
            catalog_name: "".to_string(),
            display_size: bson_type_info.fixed_bytes_length,
            fixed_prec_scale: false,
            label: field_name.clone(),
            length: bson_type_info.fixed_bytes_length,
            col_name: field_name,
            is_nullable,
            octet_length: bson_type_info.octet_length,
            precision: bson_type_info.precision,
            scale: bson_type_info.scale,
            is_searchable: bson_type_info.searchable,
            table_name: datasource_name,
            type_name: bson_type_info.type_name.to_string(),
            is_unsigned: false,
            is_updatable: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ColumnNullability {
    Nullable,
    NoNulls,
    Unknown,
}

impl ObjectSchema {
    /// Gets the nullability of a field in this schema's properties.
    /// Nullability is determined as follows:
    ///   1. If it is not present in the schema's list of properties:
    ///     - If it is required or this schema allows additional_properties,
    ///       it is unknown nullability
    ///     - Otherwise, an error is returned
    ///
    ///   2. If it is an Any schema, it is considered nullable
    ///
    ///   3. If it is a scalar schema (i.e. not Any or AnyOf):
    ///     - If its bson type is Null, it is considered nullable
    ///     - Otherwise, its nullability depends on whether it is required
    ///
    ///   4. If it is an AnyOf schema:
    ///     - If one of the component schemas in the AnyOf list is Null, it
    ///       is considered nullable
    ///     - Otherwise, its nullability depends on whether it is required
    pub fn get_field_nullability(&self, field_name: String) -> Result<ColumnNullability> {
        use json_schema::simplified::{Atomic, Schema};

        let required = self.required.contains(&field_name);

        let field_schema = self.properties.get(&field_name);

        // Case 1: field not present in properties
        if field_schema.is_none() {
            if required || self.additional_properties {
                return Ok(ColumnNullability::Unknown);
            }

            return Err(Error::UnknownColumn(field_name));
        }

        let nullable = if required {
            ColumnNullability::NoNulls
        } else {
            ColumnNullability::Nullable
        };

        match field_schema.unwrap() {
            // Case 2: field is Any schema
            Schema::Atomic(Atomic::Any) => Ok(ColumnNullability::Nullable),
            // Case 3: field is scalar/array/object schema
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Null)) => Ok(ColumnNullability::Nullable),
            Schema::Atomic(Atomic::Scalar(_))
            | Schema::Atomic(Atomic::Array(_))
            | Schema::Atomic(Atomic::Object(_)) => Ok(nullable),
            // Case 4: field is AnyOf schema
            Schema::AnyOf(any_of) => {
                for any_of_schema in any_of {
                    if *any_of_schema == Atomic::Scalar(BsonTypeName::Null) {
                        return Ok(ColumnNullability::Nullable);
                    }
                }
                Ok(nullable)
            }
        }
    }
}
