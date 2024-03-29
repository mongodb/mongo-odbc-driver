use std::collections::HashMap;

use crate::{
    json_schema::{
        simplified::{Atomic, ObjectSchema, Schema},
        BsonTypeName,
    },
    BsonTypeInfo, Error, Result, TypeMode,
};
use definitions::{Nullability, SqlCode, SqlDataType};
use itertools::Itertools;
use log::warn;
use serde::{Deserialize, Serialize};

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAttribute or SQLDescribeCol and when converting the data to the targeted C type.
#[derive(Clone, Debug)]
pub struct MongoColMetadata {
    pub base_col_name: String,
    pub base_table_name: String,
    pub case_sensitive: bool,
    pub catalog_name: String,
    // more info for column size can be found here:
    // https://learn.microsoft.com/en-us/sql/odbc/reference/appendixes/column-size?view=sql-server-ver16
    pub display_size: Option<u16>,
    pub fixed_prec_scale: bool,
    pub label: String,
    pub length: Option<u16>,
    pub literal_prefix: Option<&'static str>,
    pub literal_suffix: Option<&'static str>,
    pub col_name: String,
    pub nullability: Nullability,
    pub num_prec_radix: Option<u16>,
    pub octet_length: Option<u16>,
    pub precision: Option<u16>,
    pub scale: Option<u16>,
    pub searchable: i32,
    pub table_name: String,
    // BSON type name
    pub type_name: String,
    // Sql type integer
    pub sql_type: SqlDataType,
    // non-concise SqlDataType
    pub non_concise_type: SqlDataType,
    pub sql_code: Option<SqlCode>,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}

impl MongoColMetadata {
    pub fn new_metadata_from_bson_type_info(
        _current_db: &str,
        datasource_name: String,
        field_name: String,
        bson_type_info: BsonTypeInfo,
        type_mode: TypeMode,
        nullability: Nullability,
    ) -> MongoColMetadata {
        MongoColMetadata {
            // For base_col_name, base_table_name, and catalog_name, we do
            // not have this information in sqlGetResultSchema, so these will
            // always be empty string for now.
            base_col_name: "".to_string(),
            base_table_name: "".to_string(),
            case_sensitive: bson_type_info.is_case_sensitive,
            catalog_name: "".to_string(),
            display_size: bson_type_info.fixed_bytes_length(type_mode),
            fixed_prec_scale: bson_type_info.fixed_prec_scale,
            label: field_name.clone(),
            length: bson_type_info.fixed_bytes_length(type_mode),
            literal_prefix: bson_type_info.literal_prefix,
            literal_suffix: bson_type_info.literal_suffix,
            col_name: field_name,
            nullability,
            num_prec_radix: bson_type_info.num_prec_radix,
            octet_length: bson_type_info.octet_length(type_mode),
            precision: bson_type_info.precision(type_mode),
            scale: bson_type_info.scale,
            searchable: bson_type_info.searchable,
            table_name: datasource_name,
            type_name: bson_type_info.type_name.to_string(),
            sql_type: bson_type_info.sql_type(type_mode),
            non_concise_type: bson_type_info.non_concise_type(type_mode),
            sql_code: bson_type_info.sql_code,
            is_unsigned: bson_type_info.is_unsigned.unwrap_or(true),
            is_updatable: false,
        }
    }

    pub fn new_metadata_from_bson_type_info_default(
        current_db: &str,
        datasource_name: String,
        field_name: String,
        bson_type_info: BsonTypeInfo,
        nullability: Nullability,
    ) -> MongoColMetadata {
        Self::new_metadata_from_bson_type_info(
            current_db,
            datasource_name,
            field_name,
            bson_type_info,
            TypeMode::Standard,
            nullability,
        )
    }

    pub fn new(
        current_db: &str,
        datasource_name: String,
        field_name: String,
        field_schema: Schema,
        nullability: Nullability,
        type_mode: TypeMode,
    ) -> MongoColMetadata {
        warn!("field schema {:?}", field_schema);

        let bson_type_info: BsonTypeInfo = field_schema.into();
        warn!("bson type info {:?}", bson_type_info);

        MongoColMetadata::new_metadata_from_bson_type_info(
            current_db,
            datasource_name,
            field_name,
            bson_type_info,
            type_mode,
            nullability,
        )
    }
}

// Struct representing the response for a sqlGetResultSchema command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct SqlGetSchemaResponse {
    pub ok: i32,
    pub schema: VersionedJsonSchema,
    #[serde(rename = "selectOrder")]
    pub select_order: Option<Vec<Vec<String>>>,
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
    /// to datasources. The metadata is sorted by select order when possible, and
    /// when not, alphabetically by datasource name and then by field name.
    /// The latter case, for a result set with schema:
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
    pub(crate) fn process_result_metadata(
        &self,
        current_db: &str,
        type_mode: TypeMode,
    ) -> Result<Vec<MongoColMetadata>> {
        warn!("schema {:?}", self.schema);
        let result_set_schema: crate::json_schema::simplified::Schema =
            self.schema.json_schema.clone().try_into()?;
        warn!("simplified schema {:?}", result_set_schema);
        let result_set_object_schema = result_set_schema.assert_object_schema()?;

        // create a map from the naming convention used by select order ([datasource name, column name]),
        // to the schema
        let mut processed_result_set_metadata: HashMap<Vec<String>, MongoColMetadata> =
            result_set_object_schema
                .clone()
                // 1. Access result_set_schema.properties and turn into an iterator
                .properties
                .into_iter()
                // 2. for each datasource, convert the schema to column metadata. Then,
                //    turn the resulting vector of metadata into key-value pairs for the
                //    metadata map we are creating.
                .map(|(datasource_name, datasource_schema)| {
                    let schema = Self::schema_to_col_metadata(
                        &datasource_schema,
                        current_db,
                        &datasource_name,
                        type_mode,
                    )?;
                    warn!("schema after schema to col metadata {:?}", schema);
                    Ok(schema
                        .into_iter()
                        .map(|col| (vec![col.table_name.clone(), col.col_name.clone()], col)))
                })
                // flatten the key-value pairs representing the metadata into a single vector,
                // then finally convert to a HashMap
                .flatten_ok()
                .collect::<Result<HashMap<Vec<String>, MongoColMetadata>>>()?;

        Ok(match self.select_order {
            // in the select list order is None, for example if using an older adf version, sort
            None => processed_result_set_metadata
                .into_values()
                .sorted_by(|a, b| match Ord::cmp(&a.table_name, &b.table_name) {
                    core::cmp::Ordering::Equal => Ord::cmp(&a.col_name, &b.col_name),
                    v => v,
                })
                .collect(),
            // given a select order, convert the values of the map into an ordered vector
            _ => self
                .select_order
                .as_ref()
                .unwrap()
                .iter()
                .map(|key| processed_result_set_metadata.remove(key).unwrap())
                .collect(),
        })
    }

    /// Converts a sqlGetSchema command response into a list of column
    /// metadata. Ensures the top-level schema is an Object with properties,
    /// The metadata is sorted alphabetically by property name and then by field name.
    /// As in, a result set with schema:
    ///
    ///   {
    ///     bsonType: "object",
    ///     properties: {
    ///       "foo": {
    ///         bsonType: "int",
    ///       },
    ///       "bar": {
    ///         bsonType: "double",
    ///       }
    ///   }
    ///
    /// produces a list of metadata with the order: "bar", "foo".
    pub(crate) fn process_collection_metadata(
        &self,
        current_db: &str,
        current_collection: &str,
        type_mode: TypeMode,
    ) -> Result<Vec<MongoColMetadata>> {
        let collection_schema: crate::json_schema::simplified::Schema =
            self.schema.json_schema.clone().try_into()?;
        Self::schema_to_col_metadata(
            &collection_schema,
            current_db,
            current_collection,
            type_mode,
        )
    }

    // Helper function that asserts the the passed object_schema is actually an ObjectSchema
    // (required), and then converts all the propety schemata of the properties into a
    // Result<Vec<MongoColMetadata>>, one MongoColMetadata per property schema in lexicographical
    // order.
    fn schema_to_col_metadata(
        object_schema: &crate::json_schema::simplified::Schema,
        current_db: &str,
        current_collection: &str,
        type_mode: TypeMode,
    ) -> Result<Vec<MongoColMetadata>> {
        let object_schema = object_schema.assert_object_schema()?;

        object_schema
            // 1. Access object_schema.properties and sort alphabetically.
            //    This means we are sorting by field name. This is necessary
            //    because this defines our ordinal positions.
            .properties
            .clone()
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            // 2. Map each field into a MongoColMetadata.
            .map(|(name, schema)| {
                let field_nullability = object_schema.get_field_nullability(name.clone())?;

                Ok(MongoColMetadata::new(
                    current_db,
                    current_collection.to_string(),
                    name,
                    schema,
                    field_nullability,
                    type_mode,
                ))
            })
            .collect::<Result<Vec<_>>>()
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
                return Ok(Nullability::SQL_NULLABLE_UNKNOWN);
            }

            return Err(Error::UnknownColumn(field_name));
        }

        let nullable = if required {
            Nullability::SQL_NO_NULLS
        } else {
            Nullability::SQL_NULLABLE
        };

        match field_schema.unwrap() {
            // Case 2: field is Any schema
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Any)) => Ok(Nullability::SQL_NULLABLE),
            // Case 3: field is scalar/array/object schema
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Null))
            | Schema::Atomic(Atomic::Scalar(BsonTypeName::Undefined)) => {
                Ok(Nullability::SQL_NULLABLE)
            }
            Schema::Atomic(Atomic::Scalar(_))
            | Schema::Atomic(Atomic::Array(_))
            | Schema::Atomic(Atomic::Object(_)) => Ok(nullable),
            // Case 4: field is AnyOf schema
            Schema::AnyOf(any_of) => {
                for any_of_schema in any_of {
                    if *any_of_schema == Atomic::Scalar(BsonTypeName::Null) {
                        return Ok(Nullability::SQL_NULLABLE);
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
            map, Error, TypeMode,
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
                select_order: Some(vec![]),
            };

            let actual = input.process_result_metadata("test_db", TypeMode::Standard);

            match actual {
                Err(Error::InvalidResultSetJsonSchema(_)) => (),
                Err(e) => panic!("unexpected error: {e:?}"),
                Ok(ok) => panic!("unexpected result: {ok:?}"),
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
                select_order: Some(vec![]),
            };

            let actual = input.process_result_metadata("test_db", TypeMode::Standard);

            match actual {
                Err(Error::InvalidResultSetJsonSchema(_)) => (),
                Err(e) => panic!("unexpected error: {e:?}"),
                Ok(ok) => panic!("unexpected result: {ok:?}"),
            }
        }

        #[test]
        fn fields_sorted_alphabetical_no_select_order() {
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
                select_order: None,
            };

            let res = input.process_result_metadata("test_db", TypeMode::Standard);

            match res {
                Err(e) => panic!("unexpected error: {e:?}"),
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

        #[test]
        fn fields_sorted_out_of_order_select_order() {
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
                select_order: Some(vec![
                    vec!["foo".to_string(), "b".to_string()],
                    vec!["foo".to_string(), "a".to_string()],
                    vec!["bar".to_string(), "c".to_string()],
                ]),
            };

            let res = input.process_result_metadata("test_db", TypeMode::Standard);

            match res {
                Err(e) => panic!("unexpected error: {e:?}"),
                Ok(actual) => {
                    // There should be 3 fields
                    assert_eq!(3, actual.len());

                    for (idx, table_name, col_name) in
                        [(0, "foo", "b"), (1, "foo", "a"), (2, "bar", "c")]
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
        use definitions::Nullability;

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
            expected = Nullability::SQL_NULLABLE_UNKNOWN,
            input_schema = ObjectSchema {
                properties: map! {},
                required: set! {"a".to_string()},
                additional_properties: false,
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            field_not_in_properties_but_additional_properties_allowed,
            expected = Nullability::SQL_NULLABLE_UNKNOWN,
            input_schema = ObjectSchema {
                properties: map! {},
                required: set! {},
                additional_properties: true,
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            any_schema,
            expected = Nullability::SQL_NULLABLE,
            input_schema = ObjectSchema {
                properties: map! {
                    "a".to_string() => Schema::Atomic(Atomic::Scalar(BsonTypeName::Any))
                },
                required: set! {},
                additional_properties: false
            },
            input_field = "a".to_string()
        );

        get_field_nullability_test!(
            scalar_null_schema,
            expected = Nullability::SQL_NULLABLE,
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
            expected = Nullability::SQL_NULLABLE,
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
            expected = Nullability::SQL_NO_NULLS,
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
            expected = Nullability::SQL_NULLABLE,
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
            expected = Nullability::SQL_NULLABLE,
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
            expected = Nullability::SQL_NO_NULLS,
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
                Err(e) => panic!("unexpected error: {e:?}"),
                Ok(ok) => panic!("unexpected result: {ok:?}"),
            }
        }
    }
}
