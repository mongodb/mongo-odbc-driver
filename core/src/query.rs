use crate::bson_type_info::BsonTypeInfo;
use crate::{
    conn::MongoConnection, err::Result, json_schema, stmt::MongoStatement, BsonType, BsonTypeName,
    Error, Items, Schema,
};
use bson::{doc, Bson, Document};
use itertools::Itertools;
use mongodb::{options::AggregateOptions, sync::Cursor};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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

#[derive(Clone, Debug)]
pub enum ColumnNullability {
    Nullable,
    NoNulls,
    Unknown,
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
#[derive(Clone, Default)]
struct SimplifiedJsonSchema {
    pub bson_type: Option<BsonTypeName>,
    pub properties: Option<HashMap<String, SimplifiedJsonSchema>>,
    pub any_of: Option<Vec<SimplifiedJsonSchema>>,
    pub required: Option<BTreeSet<String>>,
    pub items: Option<Box<SimplifiedJsonSchema>>,
    pub additional_properties: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
enum AlternativeWay {
    Single(SingleJsonSchema),
    AnyOf(BTreeSet<SingleJsonSchema>)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
enum SingleJsonSchema {
    Any,
    Scalar(BsonTypeName),
    Object(ObjectSchema),
    Array(Box<AlternativeWay>)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct ObjectSchema {
    properties: BTreeMap<String, AlternativeWay>,
    required: BTreeSet<String>,
    additional_properties: bool,
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
                        field_schema,
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
        is_nullable: ColumnNullability,
    ) -> Result<MongoColMetadata> {
        let bson_type_info: BsonTypeInfo = (field_name.clone(), field_schema).try_into()?;

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
    const ANY: SimplifiedJsonSchema = SimplifiedJsonSchema {
        bson_type: None,
        properties: None,
        any_of: None,
        required: None,
        items: None,
        additional_properties: false,
    };

    /// A datasource schema must be an Object schema. Unlike Object schemata
    /// in general, the properties field cannot be null.
    fn assert_datasource_schema(&self) -> Result<()> {
        if self.bson_type == Some(BsonTypeName::Object) && self.properties.is_some() {
            Ok(())
        } else {
            Err(Error::InvalidResultSetJsonSchema)
        }
    }

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
    fn get_field_nullability(&self, field_name: String) -> Result<ColumnNullability> {
        let required =
            self.required.is_some() && self.required.as_ref().unwrap().contains(&field_name);

        let field_schema = self.properties.as_ref().unwrap().get(&field_name);

        // Case 1: field not present in properties
        if field_schema.is_none() {
            if required || self.additional_properties {
                return Ok(ColumnNullability::Unknown);
            }

            return Err(Error::UnknownColumn(field_name));
        }

        let field_schema = field_schema.unwrap();

        // Case 2: field is Any schema
        if field_schema.is_any() {
            return Ok(ColumnNullability::Nullable);
        }

        let nullable = if required {
            ColumnNullability::NoNulls
        } else {
            ColumnNullability::Nullable
        };

        // Case 3: field is scalar schema
        if field_schema.bson_type.is_some() {
            return Ok(if field_schema.bson_type == Some(BsonTypeName::Null) {
                ColumnNullability::Nullable
            } else {
                nullable
            });
        }

        // Case 4: field is AnyOf schema
        if let Some(any_of) = self.any_of.as_ref() {
            for any_of_schema in any_of {
                if any_of_schema.bson_type == Some(BsonTypeName::Null) {
                    return Ok(ColumnNullability::Nullable);
                }
            }
        } else {
            return Err(Error::InvalidResultSetJsonSchema);
        }

        Ok(nullable)
    }

    fn is_any(&self) -> bool {
        return self.bson_type.is_none()
            && self.properties.is_none()
            && self.any_of.is_none()
            && self.required.is_none()
            && self.items.is_none()
            && !self.additional_properties;
    }
}

impl AlternativeWay {

    /// A datasource schema must be an Object schema. Unlike Object schemata
    /// in general, the properties field cannot be null.
    fn assert_datasource_schema(&self) -> Result<&ObjectSchema> {
        match self {
            AlternativeWay::Single(SingleJsonSchema::Object(s)) => Ok(s),
            _ => Err(Error::InvalidResultSetJsonSchema)
        }
    }

    fn is_any(&self) -> bool {
        match self {
            AlternativeWay::Single(SingleJsonSchema::Any) => true,
            _ => false
        }
    }
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
    fn get_field_nullability(&self, field_name: String) -> Result<ColumnNullability> {
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
            AlternativeWay::Single(SingleJsonSchema::Any) => Ok(ColumnNullability::Nullable),
            // Case 3: field is scalar/array/object schema
            AlternativeWay::Single(SingleJsonSchema::Scalar(BsonTypeName::Null)) => Ok(ColumnNullability::Nullable),
            AlternativeWay::Single(SingleJsonSchema::Scalar(_)) | AlternativeWay::Single(SingleJsonSchema::Array(_)) | AlternativeWay::Single(SingleJsonSchema::Object(_)) => Ok(nullable),
            // Case 4: field is AnyOf schema
            AlternativeWay::AnyOf(any_of) => {
                for any_of_schema in any_of {
                    if *any_of_schema == SingleJsonSchema::Scalar(BsonTypeName::Null) {
                        return Ok(ColumnNullability::Nullable)
                    }
                }
                Ok(nullable)
            }
        }
    }
}

// Converts a deserialized json_schema::Schema into a SimplifiedJsonSchema. The
// SimplifiedJsonSchema instance is semantically equivalent to the base schema,
// but bson_type has to be a single type otherwise the types will get pushed
// down in the any_of list.
impl From<json_schema::Schema> for SimplifiedJsonSchema {
    fn from(schema: Schema) -> Self {
        let mut properties = schema.properties.map(|properties| {
            properties
                .into_iter()
                .map(|(field_name, field_schema)| (field_name, field_schema.into()))
                .collect::<HashMap<String, SimplifiedJsonSchema>>()
        });

        let mut any_of = schema.any_of.map(|any_of| {
            any_of
                .into_iter()
                .map(|any_of_schema| any_of_schema.into())
                .collect::<Vec<SimplifiedJsonSchema>>()
        });

        let mut required = schema
            .required
            .map(|required| required.into_iter().collect::<BTreeSet<String>>());

        let mut items = schema.items.map(|items| {
            match items {
                // The single-schema variant of the `items`
                // field constrains all elements of the array.
                Items::Single(items_schema) => Box::new(SimplifiedJsonSchema::from(*items_schema)), // actually convert
                // The multiple-schema variant of the `items` field only
                // asserts the schemas for the array items at specified
                // indexes, and imposes no constraint on items at larger
                // indexes. As such, the only schema that can describe all
                // elements of the array is `Any`.
                Items::Multiple(items_schemas) => Box::new(SimplifiedJsonSchema::ANY),
            }
        });

        let mut additional_properties = schema.additional_properties.unwrap_or(false);

        // schema.bson_type requires special handling. If it is a Single type,
        // then the new SimplifiedJsonSchema simply copies it over as its new
        // bson_type. If it is a Multiple, then we push everything down into
        // the any_of field.
        let bson_type = match schema.bson_type {
            None => None,
            Some(BsonType::Single(t)) => Some(t),
            Some(BsonType::Multiple(types)) => {
                let mut additional_any_of_schemas = types
                    .into_iter()
                    .map(|t| {
                        match t {
                            // If one of the BSON types is Array, move the items
                            // information into the schema that will be nested in
                            // the top-level any_of and set the top-level items to
                            // None.
                            BsonTypeName::Array => {
                                let nested_items = items.clone();
                                items = None;
                                SimplifiedJsonSchema {
                                    bson_type: Some(t),
                                    items: nested_items,
                                    ..Default::default()
                                }
                            }
                            // If one of the BSON types is Object, move the
                            // properties, required, and additional_properties
                            // information into the schema that will be nested
                            // in the top-level any_of and set the top-level
                            // fields to None.
                            BsonTypeName::Object => {
                                let nested_properties = properties.clone();
                                let nested_required = required.clone();
                                let nested_additional_properties = additional_properties;
                                properties = None;
                                required = None;
                                additional_properties = false;
                                SimplifiedJsonSchema {
                                    bson_type: Some(t),
                                    properties: nested_properties,
                                    required: nested_required,
                                    additional_properties: nested_additional_properties,
                                    ..Default::default()
                                }
                            }
                            // If we encounter any other BSON type, simply create
                            // a SimplifiedJsonSchema with the relevant bson_type.
                            t => SimplifiedJsonSchema {
                                bson_type: Some(t),
                                ..Default::default()
                            },
                        }
                    })
                    .collect::<Vec<SimplifiedJsonSchema>>();

                // Extend the top-level any_of with the new, additional
                // schemas created here from the Multiple BSON types.
                any_of = match any_of {
                    None => Some(additional_any_of_schemas),
                    Some(mut v) => {
                        v.append(&mut additional_any_of_schemas);
                        Some(v)
                    }
                };

                // Set the top-level bson_type to None when there are multiple
                // BSON types.
                None
            }
        };

        // Flatten nested any_ofs
        any_of = any_of.map(|v| {
            v.into_iter()
                .flat_map(|s| s.any_of.unwrap_or(vec![]))
                .collect::<Vec<SimplifiedJsonSchema>>()
        });

        SimplifiedJsonSchema {
            bson_type,
            properties,
            any_of,
            required,
            items,
            additional_properties,
        }
    }
}

impl TryFrom<json_schema::Schema> for SingleJsonSchema {
    type Error = Error;

    fn try_from(schema: Schema) -> std::result::Result<Self, Self::Error> {
        match schema {
            json_schema::Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: None,
            } => Ok(SingleJsonSchema::Any),
            json_schema::Schema {
                bson_type: Some(bson_type),
                properties,
                required,
                additional_properties,
                items,
                any_of: None,
            } => {
                match bson_type {
                    BsonType::Single(BsonTypeName::Array) => {
                        Ok(SingleJsonSchema::Array(Box::new(match items {
                            Some(Items::Single(s)) => AlternativeWay::try_from(*s)?,
                            Some(Items::Multiple(_)) => AlternativeWay::Single(SingleJsonSchema::Any),
                            None => AlternativeWay::Single(SingleJsonSchema::Any)
                        })))
                    },
                    BsonType::Single(BsonTypeName::Object) => {
                        Ok(SingleJsonSchema::Object(ObjectSchema {
                            properties: properties.unwrap_or_default().into_iter().map(|(prop, prop_schema)| Ok((prop, AlternativeWay::try_from(prop_schema)?))).collect::<Result<_>>()?,
                            required: required.unwrap_or_default().into_iter().collect(),
                            additional_properties: additional_properties.unwrap_or(true),
                        }))
                    },
                    BsonType::Single(t) => {
                        Ok(SingleJsonSchema::Scalar(t))
                    },
                    BsonType::Multiple(types) => Err(Error::InvalidResultSetJsonSchema)
                }
            },
            _ => Err(Error::InvalidResultSetJsonSchema)
        }
    }
}

impl TryFrom<json_schema::Schema> for AlternativeWay {
    type Error = Error;

    fn try_from(schema: Schema) -> std::result::Result<Self, Self::Error> {
        match schema {
            json_schema::Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: None,
            } => Ok(AlternativeWay::Single(SingleJsonSchema::Any)),
            json_schema::Schema {
                bson_type: Some(bson_type),
                properties,
                required,
                additional_properties,
                items,
                any_of: None,
            } => {
                match bson_type {
                    BsonType::Single(BsonTypeName::Array) => {
                        Ok(AlternativeWay::Single(SingleJsonSchema::Array(Box::new(match items {
                            Some(Items::Single(s)) => AlternativeWay::try_from(*s)?,
                            Some(Items::Multiple(_)) => AlternativeWay::Single(SingleJsonSchema::Any),
                            None => AlternativeWay::Single(SingleJsonSchema::Any)
                        }))))
                    },
                    BsonType::Single(BsonTypeName::Object) => {
                        Ok(AlternativeWay::Single(SingleJsonSchema::Object(ObjectSchema {
                            properties: properties.unwrap_or_default().into_iter().map(|(prop, prop_schema)| Ok((prop, AlternativeWay::try_from(prop_schema)?))).collect::<Result<_>>()?,
                            required: required.unwrap_or_default().into_iter().collect(),
                            additional_properties: additional_properties.unwrap_or(true),
                        })))
                    },
                    BsonType::Single(t) => {
                        Ok(AlternativeWay::Single(SingleJsonSchema::Scalar(t)))
                    },
                    BsonType::Multiple(types) => {
                        Ok(AlternativeWay::AnyOf(types.into_iter().map(|t| {
                            match t {
                                BsonTypeName::Array => {
                                    SingleJsonSchema::try_from(json_schema::Schema {
                                        bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                                        items: items.clone(),
                                        ..Default::default()
                                    })
                                },
                                BsonTypeName::Object => {
                                    SingleJsonSchema::try_from(json_schema::Schema {
                                        bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                                        properties: properties.clone(),
                                        required: required.clone(),
                                        additional_properties: additional_properties.clone(),
                                        ..Default::default()
                                    })
                                },
                                _ => Ok(SingleJsonSchema::Scalar(t))
                            }
                        }).collect::<Result<_>>()?))
                    }
                }
            },
            json_schema::Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: Some(any_of)
            } => Ok(AlternativeWay::AnyOf(any_of.into_iter().map(SingleJsonSchema::try_from).collect::<Result<BTreeSet<SingleJsonSchema>>>()?)),
            _ => Err(Error::InvalidResultSetJsonSchema)
        }
    }
}

impl TryFrom<(String, SimplifiedJsonSchema)> for BsonTypeInfo {
    type Error = Error;

    fn try_from(
        (field_name, field_schema): (String, SimplifiedJsonSchema),
    ) -> std::result::Result<Self, Self::Error> {
        if field_schema.bson_type.is_none() {
            return Err(Error::MissingFieldBsonType(field_name));
        }

        Ok(match field_schema.bson_type.unwrap() {
            BsonTypeName::Array => BsonTypeInfo::ARRAY,
            BsonTypeName::Object => BsonTypeInfo::OBJECT,
            BsonTypeName::Null => BsonTypeInfo::NULL,
            BsonTypeName::String => BsonTypeInfo::STRING,
            BsonTypeName::Int => BsonTypeInfo::INT,
            BsonTypeName::Double => BsonTypeInfo::DOUBLE,
            BsonTypeName::Long => BsonTypeInfo::LONG,
            BsonTypeName::Decimal => BsonTypeInfo::DECIMAL,
            BsonTypeName::BinData => BsonTypeInfo::BINDATA,
            BsonTypeName::ObjectId => BsonTypeInfo::OBJECTID,
            BsonTypeName::Bool => BsonTypeInfo::BOOL,
            BsonTypeName::Date => BsonTypeInfo::DATE,
            BsonTypeName::Regex => BsonTypeInfo::REGEX,
            BsonTypeName::DbPointer => BsonTypeInfo::DBPOINTER,
            BsonTypeName::Javascript => BsonTypeInfo::JAVASCRIPT,
            BsonTypeName::Symbol => BsonTypeInfo::SYMBOL,
            BsonTypeName::JavascriptWithScope => BsonTypeInfo::JAVASCRIPTWITHSCOPE,
            BsonTypeName::Timestamp => BsonTypeInfo::TIMESTAMP,
            BsonTypeName::MinKey => BsonTypeInfo::MINKEY,
            BsonTypeName::MaxKey => BsonTypeInfo::MAXKEY,
        })
    }
}

impl From<AlternativeWay> for BsonTypeInfo {
    fn from(v: AlternativeWay) -> Self {
        match v {
            AlternativeWay::Single(SingleJsonSchema::Any) => BsonTypeInfo::BSON,
            AlternativeWay::Single(SingleJsonSchema::Scalar(t)) => t.into(),
            AlternativeWay::Single(SingleJsonSchema::Object(_)) => BsonTypeInfo::OBJECT,
            AlternativeWay::Single(SingleJsonSchema::Array(_)) => BsonTypeInfo::ARRAY,
            AlternativeWay::AnyOf(_) => BsonTypeInfo::BSON,
        }
    }
}

impl From<BsonTypeName> for BsonTypeInfo {
    fn from(v: BsonTypeName) -> Self {
        match v {
            BsonTypeName::Array => BsonTypeInfo::ARRAY,
            BsonTypeName::Object => BsonTypeInfo::OBJECT,
            BsonTypeName::Null => BsonTypeInfo::NULL,
            BsonTypeName::String => BsonTypeInfo::STRING,
            BsonTypeName::Int => BsonTypeInfo::INT,
            BsonTypeName::Double => BsonTypeInfo::DOUBLE,
            BsonTypeName::Long => BsonTypeInfo::LONG,
            BsonTypeName::Decimal => BsonTypeInfo::DECIMAL,
            BsonTypeName::BinData => BsonTypeInfo::BINDATA,
            BsonTypeName::ObjectId => BsonTypeInfo::OBJECTID,
            BsonTypeName::Bool => BsonTypeInfo::BOOL,
            BsonTypeName::Date => BsonTypeInfo::DATE,
            BsonTypeName::Regex => BsonTypeInfo::REGEX,
            BsonTypeName::DbPointer => BsonTypeInfo::DBPOINTER,
            BsonTypeName::Javascript => BsonTypeInfo::JAVASCRIPT,
            BsonTypeName::Symbol => BsonTypeInfo::SYMBOL,
            BsonTypeName::JavascriptWithScope => BsonTypeInfo::JAVASCRIPTWITHSCOPE,
            BsonTypeName::Timestamp => BsonTypeInfo::TIMESTAMP,
            BsonTypeName::MinKey => BsonTypeInfo::MINKEY,
            BsonTypeName::MaxKey => BsonTypeInfo::MAXKEY,
        }
    }
}
