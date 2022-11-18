use crate::{
    col_metadata::MongoColMetadata,
    conn::MongoConnection,
    err::Result,
    json_schema::{
        self,
        simplified::{Atomic, ObjectSchema, Schema},
        BsonTypeName,
    },
    stmt::MongoStatement,
    Error,
};
use bson::{doc, Bson, Document};
use itertools::Itertools;
use mongodb::{options::AggregateOptions, sync::Cursor};
use odbc_sys::Nullability;
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
        query_timeout: Option<u32>,
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
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self) -> Result<bool> {
        let res = self.resultset_cursor.advance().map_err(Error::Mongo);
        if let Ok(false) = res {
            // deserialize_current unwraps None if we do not check the value of advance.
            self.current = None;
            return res;
        }
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

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }
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
    #[serde(rename = "jsonSchema")]
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

                    Ok(MongoColMetadata::new(
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
    pub fn get_field_nullability(&self, field_name: String) -> Result<Nullability> {
        let required = self.required.contains(&field_name);

        let field_schema = self.properties.get(&field_name);

        // Case 1: field not present in properties
        if field_schema.is_none() {
            if required || self.additional_properties {
                return Ok(Nullability::UNKNOWN);
            }

            return Err(Error::UnknownColumn(field_name));
        }

        let nullable = if required {
            Nullability::NO_NULLS
        } else {
            Nullability::NULLABLE
        };

        match field_schema.unwrap() {
            // Case 2: field is Any schema
            Schema::Atomic(Atomic::Any) => Ok(Nullability::NULLABLE),
            // Case 3: field is scalar/array/object schema
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Null))
            | Schema::Atomic(Atomic::Scalar(BsonTypeName::Undefined)) => Ok(Nullability::NULLABLE),
            Schema::Atomic(Atomic::Scalar(_))
            | Schema::Atomic(Atomic::Array(_))
            | Schema::Atomic(Atomic::Object(_)) => Ok(nullable),
            // Case 4: field is AnyOf schema
            Schema::AnyOf(any_of) => {
                for any_of_schema in any_of {
                    if *any_of_schema == Atomic::Scalar(BsonTypeName::Null) {
                        return Ok(Nullability::NULLABLE);
                    }
                }
                Ok(nullable)
            }
        }
    }
}

#[cfg(test)]
mod unit {
    mod process_metadata {
        use crate::{
            json_schema::{BsonType, BsonTypeName, Schema},
            map,
            query::{SqlGetResultSchemaResponse, VersionedJsonSchema},
            Error,
        };

        #[test]
        fn top_level_schema_not_object() {
            let input = SqlGetResultSchemaResponse {
                ok: 1,
                schema: VersionedJsonSchema {
                    version: 1,
                    json_schema: Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    },
                },
            };

            let actual = input.process_metadata("test_db");

            match actual {
                Err(Error::InvalidResultSetJsonSchema) => (),
                Err(e) => panic!("unexpected error: {:?}", e),
                Ok(ok) => panic!("unexpected result: {:?}", ok),
            }
        }

        #[test]
        fn property_schema_not_object() {
            let input = SqlGetResultSchemaResponse {
                ok: 1,
                schema: VersionedJsonSchema {
                    version: 1,
                    json_schema: Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                        properties: Some(map! {
                            "a".to_string() => Schema {
                                bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                                ..Default::default()
                            }
                        }),
                        ..Default::default()
                    },
                },
            };

            let actual = input.process_metadata("test_db");

            match actual {
                Err(Error::InvalidResultSetJsonSchema) => (),
                Err(e) => panic!("unexpected error: {:?}", e),
                Ok(ok) => panic!("unexpected result: {:?}", ok),
            }
        }

        #[test]
        fn fields_sorted_alphabetically() {
            let input = SqlGetResultSchemaResponse {
                ok: 1,
                schema: VersionedJsonSchema {
                    version: 1,
                    json_schema: Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                        properties: Some(map! {
                            "foo".to_string() => Schema {
                                bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                                properties: Some(map! {
                                    "b".to_string() => Schema {
                                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                                        ..Default::default()
                                    },
                                    "a".to_string() => Schema {
                                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                                        ..Default::default()
                                    }
                                }),
                                ..Default::default()
                            },
                            "bar".to_string() => Schema {
                                bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                                properties: Some(map! {
                                    "c".to_string() => Schema {
                                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                                        ..Default::default()
                                    }
                                }),
                                ..Default::default()
                            }
                        }),
                        ..Default::default()
                    },
                },
            };

            let res = input.process_metadata("test_db");

            match res {
                Err(e) => panic!("unexpected error: {:?}", e),
                Ok(actual) => {
                    // There should be 3 fields
                    assert_eq!(3, actual.len());

                    for (idx, table_name, col_name) in
                        [(0, "bar", "c"), (1, "foo", "a"), (2, "foo", "b")]
                    {
                        let md = &actual[idx];
                        assert_eq!(
                            (table_name, col_name),
                            (md.table_name.as_str(), md.col_name.as_str())
                        )
                    }
                }
            }
        }
    }

    mod object_schema {
        use crate::{
            json_schema::{
                simplified::{Atomic, ObjectSchema, Schema},
                BsonTypeName,
            },
            map, set, Error,
        };
        use odbc_sys::Nullability;

        macro_rules! get_field_nullability_test {
            ($func_name:ident, expected = $expected:expr, input_schema = $input_schema:expr, input_field = $input_field:expr) => {
                #[test]
                fn $func_name() {
                    let actual = $input_schema.get_field_nullability($input_field).unwrap();
                    assert_eq!($expected, actual)
                }
            };
        }

        get_field_nullability_test!(
            field_not_in_properties_but_is_required,
            expected = Nullability::UNKNOWN,
            input_schema = ObjectSchema {
                properties: map! {},
                required: set! {"a".to_string()},
                additional_properties: false,
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            field_not_in_properties_but_additional_properties_allowed,
            expected = Nullability::UNKNOWN,
            input_schema = ObjectSchema {
                properties: map! {},
                required: set! {},
                additional_properties: true,
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            any_schema,
            expected = Nullability::NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::Atomic(Atomic::Any)
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            scalar_null_schema,
            expected = Nullability::NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Null))
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            nonrequired_scalar_nonnull_schema,
            expected = Nullability::NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Int))
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            required_scalar_nonnull_schema,
            expected = Nullability::NO_NULLS,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Int))
                },
                required: set! {"a".to_string()},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            any_of_schema_with_null,
            expected = Nullability::NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::AnyOf(set! {Atomic::Scalar(BsonTypeName::Int), Atomic::Scalar(BsonTypeName::Null)})
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            nonrequired_any_of_schema_without_null,
            expected = Nullability::NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::AnyOf(set! {Atomic::Scalar(BsonTypeName::Int), Atomic::Scalar(BsonTypeName::String)})
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            required_any_of_schema_without_null,
            expected = Nullability::NO_NULLS,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::AnyOf(set! {Atomic::Scalar(BsonTypeName::Int), Atomic::Scalar(BsonTypeName::String)})
                },
                required: set! {"a".to_string()},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        #[test]
        fn invalid_object_schema() {
            let input_schema = ObjectSchema {
                properties: map! {},
                required: set! {},
                additional_properties: false,
            };

            let nullability = input_schema.get_field_nullability("a".to_string());
            match nullability {
                Err(Error::UnknownColumn(_)) => (),
                Err(e) => panic!("unexpected error: {:?}", e),
                Ok(ok) => panic!("unexpected result: {:?}", ok),
            }
        }
    }
}
