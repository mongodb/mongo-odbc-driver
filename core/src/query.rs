use crate::bson_type_info::BsonTypeInfo;
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
        match &client.current_db {
            None => Err(Error::NoDatabase),
            Some(current_db) => {
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
        }
    }

    // Return the number of fields/columns in the resultset
    fn _get_col_count(&self) -> u32 {
        self.resultset_metadata.len() as u32
    }

    // Get the metadata for the column with the given index.
    fn get_col_metadata(&self, col_index: u16) -> Result<&MongoColMetadata> {
        self.resultset_metadata
            .get(col_index as usize)
            .map_or(Err(Error::ColIndexOutOfBounds(col_index)), |md| Ok(md))
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
    pub is_nullable: bool,
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

// A simplified JSON Schema, relative to the json_schema::Schema struct.
// An instance of SimplifiedJsonSchema is semantically equivalent to its
// corresponding json_schema::Schema, but with two main simplifications.
// 1. The bson_type has to be a single type. If the json_schema::Schema
// contains multiple bson_types, they are pushed down into the any_of list.
// 2. The any_of is flattened.
#[derive(Clone)]
struct SimplifiedJsonSchema {
    pub bson_type: Option<String>,
    pub properties: Option<HashMap<String, Box<SimplifiedJsonSchema>>>,
    pub any_of: Option<Vec<Box<SimplifiedJsonSchema>>>,
    pub required: Option<BTreeSet<String>>,
    pub items: Option<Box<SimplifiedJsonSchema>>,
    pub additional_properties: Option<bool>,
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
    fn process_metadata(&self, current_db: &String) -> Result<Vec<MongoColMetadata>> {
        let result_set_schema: SimplifiedJsonSchema = self.schema.json_schema.clone().into();
        result_set_schema.assert_datasource_schema()?;

        result_set_schema
            // 1. Access result_set_schema.properties and sort alphabetically.
            //    This means we are sorting by datasource name.
            .properties
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            // 2. Flat-map fields for each datasource, sorting fields alphabetically.
            .flat_map(|(datasource_name, datasource_schema)| {
                datasource_schema
                    .clone()
                    .properties
                    // Since we are flat-mapping, we cannot conveniently return
                    // a Result from this closure.  Therefore, we do not assert
                    // that the datasource_schema is valid at this time, saving
                    // that for the follow-up map. We proceed by assuming there
                    // are properties,  or by using an empty HashMap when there
                    // are not.  This is equivalent to handling "no properties"
                    // without the need to return an error at this stage.
                    .unwrap_or_else(|| HashMap::default())
                    .into_iter()
                    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                    .map(move |(field_name, field_schema)| {
                        (
                            datasource_name.clone(),
                            datasource_schema.clone(),
                            field_name.clone(),
                            field_schema.clone(),
                        )
                    })
            })
            // 3. Map each field into a MongoColMetadata.
            .map(
                |(datasource_name, datasource_schema, field_name, field_schema)| {
                    datasource_schema.assert_datasource_schema()?;

                    let field_nullability =
                        datasource_schema.get_field_nullability(field_name.clone())?;

                    Self::create_column_metadata(
                        current_db,
                        datasource_name,
                        field_name,
                        *field_schema,
                        field_nullability,
                    )
                },
            )
            // 4. Collect as a Vec.
            .collect::<Result<Vec<MongoColMetadata>>>()
    }

    fn create_column_metadata(
        current_db: &String,
        datasource_name: String,
        field_name: String,
        field_schema: SimplifiedJsonSchema,
        is_nullable: bool,
    ) -> Result<MongoColMetadata> {
        let bson_type_info: BsonTypeInfo = field_schema.try_into()?;

        Ok(MongoColMetadata {
            // For base_col_name and base_table_name, we do not have this
            // information in sqlGetResultSchema, so this will always be
            // empty string.
            base_col_name: "".to_string(),
            base_table_name: "".to_string(),
            // For catalog_name, we do not have this information in
            // sqlGetResultSchema, so this will always be current_db. This
            // is not correct for correct for fields from tables in other
            // databases as part of cross-db lookups, but this is the best
            // we can do for now.
            catalog_name: current_db.clone(),
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
            table_name: datasource_name.clone(),
            type_name: bson_type_info.type_name.to_string(),
            is_unsigned: false,
            is_updatable: false,
        })
    }
}

impl SimplifiedJsonSchema {
    /// A datasource schema must be an Object schema. Unlike Object schemata
    /// in general, the properties field cannot be null.
    fn assert_datasource_schema(&self) -> Result<()> {
        if self.bson_type == Some("object".to_string()) && self.properties.is_some() {
            Ok(())
        } else {
            Err(Error::InvalidResultSetJsonSchema)
        }
    }

    /// Gets the nullability of a field in this schema's properties.
    /// Nullability is determined as follows:
    ///   - If it is not present in the schema's list of properties:
    ///     - If it is required or this schema allows additional_properties,
    ///       it is unknown nullability
    ///     - Otherwise, an error is returned
    ///
    ///   - If it is a scalar schema (i.e. not Any or AnyOf):
    ///     - If its bson type is Null, it is considered nullable
    ///     - Otherwise, its nullability depends on whether it is required
    ///
    ///   - If it is an Any schema, it is considered nullable
    ///
    ///   - If it is an AnyOf schema:
    ///     - If one of the component schemas in the AnyOf list is Null, it
    ///       is considered nullable
    ///     - Otherwise, its nullability depends on whether it is required
    fn get_field_nullability(self, _field_name: String) -> Result<bool> {
        todo!()
    }
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

impl TryFrom<SimplifiedJsonSchema> for BsonTypeInfo {
    type Error = Error;

    fn try_from(value: SimplifiedJsonSchema) -> std::result::Result<Self, Self::Error> {
        todo!()
    }
}
