use crate::{
    json_schema::{
        simplified::{Atomic, ObjectSchema, Schema},
        BsonTypeName,
    },
    BsonTypeInfo, Error, Result,
};
use itertools::Itertools;
use odbc_sys::{Nullability, SqlDataType};
use serde::{Deserialize, Serialize};

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAttribute or SQLDescribeCol and when converting the data to the targeted C type.
#[derive(Clone, Debug)]
pub struct MongoColMetadata {
    pub base_col_name: String,
    pub base_table_name: String,
    pub catalog_name: String,
    // more info for column size can be found here:
    // https://learn.microsoft.com/en-us/sql/odbc/reference/appendixes/column-size?view=sql-server-ver16
    pub display_size: Option<u16>,
    pub fixed_prec_scale: bool,
    pub label: String,
    pub length: Option<u16>,
    pub col_name: String,
    pub nullability: Nullability,
    pub octet_length: Option<u16>,
    pub precision: Option<u16>,
    pub scale: Option<u16>,
    pub is_searchable: bool,
    pub table_name: String,
    // BSON type name
    pub type_name: String,
    // Sql type integer
    pub sql_type: SqlDataType,
    // non-concise SqlDataType
    pub non_concise_type: SqlDataType,
    // sql_code, always NULL or SQL_CODE_TIMESTAMP (3) for our types
    // odbc_sys does not define this enum yet, so we just use an i32.
    pub sql_code: Option<i32>,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}

impl MongoColMetadata {
    pub fn new(
        _current_db: &str,
        datasource_name: String,
        field_name: String,
        field_schema: Schema,
        nullability: Nullability,
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
            nullability,
            octet_length: bson_type_info.octet_length,
            precision: bson_type_info.precision,
            scale: bson_type_info.scale,
            is_searchable: bson_type_info.searchable,
            table_name: datasource_name,
            type_name: bson_type_info.type_name.to_string(),
            sql_type: bson_type_info.sql_type,
            non_concise_type: match bson_type_info.sql_type {
                SqlDataType::TIMESTAMP => SqlDataType::DATETIME,
                x => x,
            },
            sql_code: match bson_type_info.sql_type {
                SqlDataType::TIMESTAMP => Some(3),
                _ => None,
            },
            is_unsigned: false,
            is_updatable: false,
        }
    }
}

// Struct representing the response for a sqlGetResultSchema command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct SqlGetSchemaResponse {
    pub ok: i32,
    pub schema: VersionedJsonSchema,
}

// Auxiliary struct representing part of the response for a sqlGetResultSchema
// command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct VersionedJsonSchema {
    pub version: i32,
    #[serde(rename = "jsonSchema")]
    pub json_schema: crate::json_schema::Schema,
}

impl SqlGetSchemaResponse {
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
    pub(crate) fn process_metadata(&self, current_db: &str) -> Result<Vec<MongoColMetadata>> {
        let result_set_schema: crate::json_schema::simplified::Schema =
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
            col_metadata::{SqlGetSchemaResponse, VersionedJsonSchema},
            json_schema::{BsonType, BsonTypeName, Schema},
            map, Error,
        };

        #[test]
        fn top_level_schema_not_object() {
            let input = SqlGetSchemaResponse {
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
            let input = SqlGetSchemaResponse {
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
            let input = SqlGetSchemaResponse {
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
