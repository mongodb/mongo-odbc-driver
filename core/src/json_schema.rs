use crate::{BsonTypeInfo, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bson_type: Option<BsonType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Items>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
}

impl Schema {
    // Remove multiple recursively removes Multiple Bson Type entries.
    pub fn remove_multiple(mut self) -> Result<Self, Error> {
        // it is invalid for both any_of and bson_type to be set because
        // any_of is a top level constructor.
        if self.bson_type.is_some() && self.any_of.is_some() {
            return Err(Error::InvalidResultSetJsonSchema(
                "Schema with bsonType and anyOf both defined is invalid",
            ));
        }
        if let Some(props) = self.properties {
            self.properties = Some(
                props
                    .into_iter()
                    .map(|(k, x)| Ok((k, x.remove_multiple()?)))
                    .collect::<Result<_, Error>>()?,
            );
        }
        if let Some(items) = self.items {
            match items {
                Items::Single(s) => {
                    self.items = Some(Items::Single(Box::new((*s).remove_multiple()?)));
                }
                // The multiple-schema variant of the `items`
                // field only asserts the schemas for the
                // array items at specified indexes, and
                // imposes no constraint on items at larger
                // indexes. As such, the only schema that can
                // describe all elements of the array is `Any`.
                Items::Multiple(_) => {
                    self.items = Some(Items::Single(Box::new(Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Any)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: None,
                        any_of: None,
                    })));
                }
            }
        }
        // Now that we have remove_multiple called in any items or properties fields, we can
        // convert Multiple bson_type into an any_of. If we did that first we would need
        // to simplify the properties and items potentially many times!
        Ok(self.multiple_to_any_of())
    }

    fn multiple_to_any_of(mut self) -> Self {
        if let Some(BsonType::Multiple(v)) = self.bson_type.as_ref() {
            // We don't want to generate an AnyOf with only one variant, just convert
            if v.len() == 1 {
                self.bson_type = Some(BsonType::Single(v[0]));
                return self;
            }
            self.any_of = Some(
                v.iter()
                    .map(|x| match x {
                        // properties, required, and additional_properties only make sense for
                        // Object
                        BsonTypeName::Object => {
                            Schema {
                                bson_type: Some(BsonType::Single(*x)),
                                properties: self.properties.clone(),
                                required: self.required.clone(),
                                additional_properties: self.additional_properties,
                                items: None,
                                any_of: None, // must be None due to assert
                            }
                        }
                        // items only make sense for Array
                        BsonTypeName::Array => {
                            Schema {
                                bson_type: Some(BsonType::Single(*x)),
                                properties: None,
                                required: None,
                                additional_properties: None,
                                items: self.items.clone(),
                                any_of: None, // must be None due to assert
                            }
                        }
                        // No fields make sense for atomic types besides bson_type.
                        _ => {
                            Schema {
                                bson_type: Some(BsonType::Single(*x)),
                                properties: None,
                                required: None,
                                additional_properties: None,
                                items: None,
                                any_of: None, // must be None due to assert
                            }
                        }
                    })
                    .collect(),
            );
            self.bson_type = None;
            self.properties = None;
            self.required = None;
            self.additional_properties = None;
            self.items = None;
        }
        self
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(untagged)]
pub enum BsonType {
    Single(BsonTypeName),
    Multiple(Vec<BsonTypeName>),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BsonTypeName {
    Object,
    Array,
    Null,
    String,
    Int,
    Double,
    Long,
    Decimal,
    BinData,
    ObjectId,
    Bool,
    Date,
    Regex,
    DbPointer,
    Javascript,
    Symbol,
    JavascriptWithScope,
    Timestamp,
    MinKey,
    MaxKey,
    Undefined,
    Any,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum Items {
    Single(Box<Schema>),
    Multiple(Vec<Schema>),
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
            BsonTypeName::Decimal => BsonTypeInfo::MONGO_DECIMAL,
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
            BsonTypeName::Undefined => BsonTypeInfo::UNDEFINED,
            BsonTypeName::Any => BsonTypeInfo::BSON,
        }
    }
}

pub mod simplified {

    use crate::{
        err::Result,
        json_schema::{self, BsonType, BsonTypeName, Items},
        BsonTypeInfo, Error,
    };
    use std::collections::{BTreeMap, BTreeSet};

    // A simplified JSON Schema, relative to the json_schema::Schema struct.
    // An instance of json_schema::simplified::Schema is semantically equivalent
    // to its corresponding json_schema::Schema with two main simplifications:
    //   1. The bson_type has to be a single type. If the json_schema::Schema
    //      contains multiple bson_types, it is represented as an AnyOf.
    //   2. There are no nested AnyOfs.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
    pub enum Schema {
        Atomic(Atomic),
        AnyOf(BTreeSet<Atomic>),
    }

    // Any non-AnyOf JsonSchema. This type enables
    // simplified::Schema to disallow nested AnyOfs.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
    pub enum Atomic {
        Scalar(BsonTypeName),
        Object(ObjectSchema),
        Array(Box<Schema>),
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
    pub struct ObjectSchema {
        pub properties: BTreeMap<String, Schema>,
        pub required: BTreeSet<String>,
        pub additional_properties: bool,
    }

    impl Schema {
        /// Assert that a given Schema is an Object, and return the resulting
        /// ObjectSchema
        pub fn assert_object_schema(&self) -> Result<&ObjectSchema> {
            match self {
                Schema::Atomic(Atomic::Object(s)) => Ok(s),
                _ => Err(Error::InvalidResultSetJsonSchema(
                    "Result set metadata JSON schema must be object with properties",
                )),
            }
        }

        pub fn is_any(&self) -> bool {
            matches!(self, Schema::Atomic(Atomic::Scalar(BsonTypeName::Any)))
        }
    }

    // Converts a deserialized json_schema::Schema into a simplified::Atomic.
    // The Atomic instance is semantically equivalent to the base schema
    // when the base schema represents any valid schema other than an AnyOf.
    // To convert a possible AnyOf base schema, use the TryFrom implementation
    // for simplified::Schema.
    impl TryFrom<json_schema::Schema> for Atomic {
        type Error = Error;

        // It is an error for the schema to contain Multiple. This function will return
        // an error if that happens.
        fn try_from(schema: json_schema::Schema) -> std::result::Result<Self, Self::Error> {
            match schema {
                json_schema::Schema {
                    bson_type: None,
                    properties: None,
                    required: None,
                    additional_properties: None | Some(false),
                    items: None,
                    any_of: None,
                } => Ok(Atomic::Scalar(BsonTypeName::Any)),
                json_schema::Schema {
                    bson_type: Some(bson_type),
                    properties,
                    required,
                    additional_properties,
                    items,
                    any_of: None,
                } => match bson_type {
                    BsonType::Single(BsonTypeName::Array) => {
                        Ok(Atomic::Array(Box::new(match items {
                            Some(Items::Single(s)) => Schema::try_from(*s)?,
                            None => Schema::Atomic(Atomic::Scalar(BsonTypeName::Any)),
                            _ => Err(Error::InvalidResultSetJsonSchema(
                                "Multiple bsonType found in Atomic context",
                            ))?,
                        })))
                    }
                    BsonType::Single(BsonTypeName::Object) => Ok(Atomic::Object(ObjectSchema {
                        properties: properties
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(prop, prop_schema)| Ok((prop, Schema::try_from(prop_schema)?)))
                            .collect::<Result<_>>()?,
                        required: required.unwrap_or_default().into_iter().collect(),
                        additional_properties: additional_properties.unwrap_or(true),
                    })),
                    BsonType::Single(t) => Ok(Atomic::Scalar(t)),
                    BsonType::Multiple(_) => Err(Error::InvalidResultSetJsonSchema(
                        "Multiple bsonType found in Atomic context",
                    )),
                },
                _ => Err(Error::InvalidResultSetJsonSchema(
                    "Invalid schema for conversion to Atomic",
                )),
            }
        }
    }

    // Converts a deserialized json_schema::Schema into a simplified::Schema.
    // The simplified::Schema instance is semantically equivalent to the base
    // schema, but bson_type has to be a single type otherwise the schema is
    // represented as an AnyOf.
    impl TryFrom<json_schema::Schema> for Schema {
        type Error = Error;

        fn try_from(schema: json_schema::Schema) -> std::result::Result<Self, Self::Error> {
            let schema = schema.remove_multiple()?;
            match schema.any_of {
                None => Ok(Schema::Atomic(schema.try_into()?)),
                Some(any_of) => match any_of.len() {
                    0 => Err(Error::InvalidResultSetJsonSchema("Empty anyOf is invalid")),
                    // AnyOf with a single schema is equivalent to the schema.
                    1 => Schema::try_from(any_of.into_iter().next().unwrap()),
                    _ => Ok(Schema::AnyOf(
                        any_of
                            .into_iter()
                            .map(Atomic::try_from)
                            .collect::<Result<BTreeSet<Atomic>>>()?,
                    )),
                },
            }
        }
    }

    impl From<Atomic> for BsonTypeInfo {
        fn from(a: Atomic) -> Self {
            match a {
                Atomic::Scalar(t) => t.into(),
                Atomic::Object(_) => BsonTypeInfo::OBJECT,
                Atomic::Array(_) => BsonTypeInfo::ARRAY,
            }
        }
    }

    impl From<Schema> for BsonTypeInfo {
        fn from(v: Schema) -> Self {
            match v {
                Schema::Atomic(a) => a.into(),
                Schema::AnyOf(b) => {
                    if b.len() == 2 {
                        let atomics = b
                            .into_iter()
                            .filter(|a| !matches!(a, Atomic::Scalar(BsonTypeName::Null)))
                            .collect::<Vec<Atomic>>();
                        if atomics.len() == 1 {
                            atomics.first().unwrap().to_owned().into()
                        } else {
                            BsonTypeInfo::BSON
                        }
                    } else {
                        BsonTypeInfo::BSON
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod unit {
    macro_rules! remove_multiple_test {
        ($func_name:ident, expected = $expected:expr, input = $input:expr) => {
            #[test]
            fn $func_name() {
                let res = $input.remove_multiple().unwrap();

                assert_eq!(res, $expected);
            }
        };
    }

    mod remove_multiple {
        use crate::{
            json_schema::{BsonType, BsonTypeName, Items, Schema},
            map,
        };

        remove_multiple_test!(
            multiple_atomic,
            expected = Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: Some(vec![
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    },
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: None,
                        any_of: None,
                    }
                ]),
            },
            input = Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Int,
                    BsonTypeName::Null
                ])),
                properties: None,
                required: None,
                additional_properties: Some(false),
                items: None,
                any_of: None,
            }
        );

        remove_multiple_test!(
            false_multiple,
            expected = Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                properties: Some(map! {
                    "x".into() => Schema::default(),
                    "y".into() => Schema::default(),
                }),
                required: Some(vec!["x".into(), "y".into()]),
                additional_properties: Some(false),
                items: None,
                any_of: None,
            },
            input = Schema {
                bson_type: Some(BsonType::Multiple(vec![BsonTypeName::Object,])),
                properties: Some(map! {
                    "x".into() => Schema::default(),
                    "y".into() => Schema::default(),
                }),
                required: Some(vec!["x".into(), "y".into()]),
                additional_properties: Some(false),
                items: None,
                any_of: None,
            }
        );

        remove_multiple_test!(
            object_and_other,
            expected = Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: Some(vec![
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                        properties: Some(map! {
                            "x".into() => Schema::default(),
                            "y".into() => Schema::default(),
                        }),
                        required: Some(vec!["x".into(), "y".into()]),
                        additional_properties: Some(false),
                        items: None,
                        any_of: None,
                    },
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: None,
                        any_of: None,
                    }
                ]),
            },
            input = Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Object,
                    BsonTypeName::Null
                ])),
                properties: Some(map! {
                    "x".into() => Schema::default(),
                    "y".into() => Schema::default(),
                }),
                required: Some(vec!["x".into(), "y".into()]),
                additional_properties: Some(false),
                items: None,
                any_of: None,
            }
        );

        remove_multiple_test!(
            array_and_other,
            expected = Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: Some(vec![
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: Some(Items::Single(Box::default())),
                        any_of: None,
                    },
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: None,
                        any_of: None,
                    }
                ]),
            },
            input = Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Array,
                    BsonTypeName::Null
                ])),
                properties: None,
                required: None,
                additional_properties: None,
                items: Some(Items::Single(Box::default())),
                any_of: None,
            }
        );

        remove_multiple_test!(
            nested,
            expected = Schema {
                bson_type: None,
                properties: None,
                required: None,
                additional_properties: None,
                items: None,
                any_of: Some(vec![
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: Some(Items::Single(Schema {
                            bson_type: None,
                            properties: None,
                            required: None,
                            additional_properties: None,
                            items: None,
                            any_of: Some(vec![Schema {
                                 bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                                 properties: Some(map! {
                                     "x".into() => Schema {
                                         bson_type: None,
                                         properties: None,
                                         required: None,
                                         additional_properties: None,
                                         items: None,
                                         any_of: Some(vec![
                                             Schema {
                                                 bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                                                 properties: Some(map!{
                                                     "b".into() => Schema::default(),
                                                     "a".into() => Schema::default(),
                                                 }),
                                                 required: Some(vec!["a".into()]),
                                                 additional_properties: Some(false),
                                                 items: None,
                                                 any_of: None
                                             },
                                             Schema {
                                                 bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                                                 properties: None,
                                                 required: None,
                                                 additional_properties: None,
                                                 items: None,
                                                 any_of: None
                                             }
                                         ])
                                     },
                                     "y".into() => Schema::default()
                                 }),
                                 required: Some(vec!["x".into(), "y".into()]),
                                 additional_properties: Some(false),
                                 items: None,
                                    any_of: None
                                 },
                                 Schema {
                                     bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                                     properties: None,
                                     required: None,
                                     additional_properties: None,
                                     items: None,
                                     any_of: None
                                }
                            ])
                        }.into())),
                        any_of: None
                    },
                    Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                        properties: None,
                        required: None,
                        additional_properties: None,
                        items: None,
                        any_of: None
                    }
                ])
            },
            input = Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Array,
                    BsonTypeName::Null
                ])),
                properties: None,
                required: None,
                additional_properties: None,
                items: Some(Items::Single(Box::new(Schema {
                    bson_type: Some(BsonType::Multiple(vec![
                        BsonTypeName::Object,
                        BsonTypeName::Null
                    ])),
                    properties: Some(map! {
                    "x".into() =>
                        Schema {
                            bson_type: Some(BsonType::Multiple(vec![
                                BsonTypeName::Object,
                                BsonTypeName::Null
                            ])),
                            properties: Some(map! {
                                "a".into() => Schema::default(),
                                "b".into() => Schema::default(),
                            }),
                            additional_properties: Some(false),
                            required: Some(vec!["a".into()]),
                            items: None,
                            any_of: None,
                        },
                        "y".into() => Schema::default(),
                    }),
                    required: Some(vec!["x".into(), "y".into()]),
                    additional_properties: Some(false),
                    items: None,
                    any_of: None,
                }))),
                any_of: None,
            }
        );
    }
    // Testing TryFrom<json_schema::Schema> for json_schema::simplified::Atomic
    macro_rules! try_from_test {
        ($func_name:ident, variant = $variant:ident, expected = $expected:expr, input = $input:expr) => {
            #[test]
            fn $func_name() {
                let res = $variant::try_from($input);

                // crate::Error cannot properly derive or implement PartialEq,
                // so we instead manually assert the expected Result.
                match (res, $expected) {
                    (Ok(actual), Ok(expected)) => assert_eq!(expected, actual),
                    (Ok(actual), Err(_)) => {
                        panic!("expected error but got result: {:?}", actual)
                    }
                    (Err(e), Ok(_)) => panic!("expected result but got error: {:?}", e),
                    (
                        Err(Error::InvalidResultSetJsonSchema(_)),
                        Err(Error::InvalidResultSetJsonSchema(_)),
                    ) => (),
                    (Err(e_actual), Err(e_expected)) => panic!(
                        "unexpected error: actual = {:?}, expected = {:?}",
                        e_actual, e_expected
                    ),
                }
            }
        };
    }

    // Testing TryFrom<json_schema::Schema> for json_schema::simplified::Atomic
    mod atomic {
        use crate::{
            json_schema::{
                self,
                simplified::{self, Atomic, ObjectSchema},
                BsonType, BsonTypeName, Items,
            },
            map, set, Error,
        };

        try_from_test!(
            any_schema,
            variant = Atomic,
            expected = Ok(Atomic::Scalar(BsonTypeName::Any)),
            input = json_schema::Schema::default()
        );

        try_from_test!(
            invalid_if_any_of_is_set,
            variant = Atomic,
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema("")),
            input = json_schema::Schema {
                any_of: Some(vec![json_schema::Schema::default()]),
                ..Default::default()
            }
        );

        try_from_test!(
            bson_type_must_not_be_none,
            variant = Atomic,
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema("")),
            input = json_schema::Schema {
                bson_type: None,
                required: Some(vec!["a".to_string()]),
                additional_properties: Some(true),
                ..Default::default()
            }
        );

        try_from_test!(
            bson_type_must_not_contain_multiple_types,
            variant = Atomic,
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema("")),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Bool,
                    BsonTypeName::Int
                ])),
                ..Default::default()
            }
        );

        try_from_test!(
            bson_type_may_be_single_type,
            variant = Atomic,
            expected = Ok(Atomic::Scalar(BsonTypeName::Bool)),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Bool)),
                ..Default::default()
            }
        );

        try_from_test!(
            array_with_no_items_simplifies_to_array_of_any,
            variant = Atomic,
            expected = Ok(Atomic::Array(Box::new(simplified::Schema::Atomic(
                Atomic::Scalar(BsonTypeName::Any)
            )))),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                ..Default::default()
            }
        );

        try_from_test!(
            array_with_single_items_simplifies_to_array_of_that_single_type,
            variant = Atomic,
            expected = Ok(Atomic::Array(Box::new(simplified::Schema::Atomic(
                Atomic::Scalar(BsonTypeName::Int)
            )))),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                items: Some(Items::Single(Box::new(json_schema::Schema {
                    bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                    ..Default::default()
                }))),
                ..Default::default()
            }
        );

        try_from_test!(
            default_object,
            variant = Atomic,
            expected = Ok(Atomic::Object(ObjectSchema {
                properties: map! {},
                required: set! {},
                additional_properties: true
            })),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                ..Default::default()
            }
        );

        try_from_test!(
            object_retains_simplified_input_values,
            variant = Atomic,
            expected = Ok(Atomic::Object(ObjectSchema {
                properties: map! {
                    "a".to_string() => simplified::Schema::Atomic(Atomic::Scalar(BsonTypeName::Int))
                },
                required: set! {"a".to_string()},
                additional_properties: false
            })),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Object)),
                properties: Some(map! {
                    "a".to_string() => json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    }
                }),
                required: Some(set! {"a".to_string()}),
                additional_properties: Some(false),
                ..Default::default()
            }
        );
    }

    // Testing TryFrom<json_schema::Schema> for json_schema::simplified::Schema
    // omitting Atomic variants in favor of the unit tests above
    mod schema {
        use crate::{
            json_schema::{
                self,
                simplified::{Atomic, Schema},
                BsonType, BsonTypeName,
            },
            set, Error,
        };

        try_from_test!(
            any_schema,
            variant = Schema,
            expected = Ok(Schema::Atomic(Atomic::Scalar(BsonTypeName::Any))),
            input = json_schema::Schema::default()
        );

        try_from_test!(
            bson_type_none_and_additional_properties_false_results_in_any,
            variant = Schema,
            expected = Ok(Schema::Atomic(Atomic::Scalar(BsonTypeName::Any))),
            input = json_schema::Schema {
                additional_properties: Some(false),
                ..Default::default()
            }
        );

        try_from_test!(
            missing_bson_type_with_other_fields_is_invalid,
            variant = Schema,
            expected = Err::<Schema, Error>(Error::InvalidResultSetJsonSchema("")),
            input = json_schema::Schema {
                additional_properties: Some(true),
                ..Default::default()
            }
        );

        try_from_test!(
            valid_schema,
            variant = Schema,
            expected = Ok(Schema::AnyOf(
                set! {Atomic::Scalar(BsonTypeName::Int), Atomic::Scalar(BsonTypeName::String)}
            )),
            input = json_schema::Schema {
                any_of: Some(vec![
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    },
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::String)),
                        ..Default::default()
                    }
                ]),
                ..Default::default()
            }
        );

        try_from_test!(
            any_of_one,
            variant = Schema,
            expected = Ok(Schema::Atomic(Atomic::Scalar(BsonTypeName::Int))),
            input = json_schema::Schema {
                any_of: Some(vec![json_schema::Schema {
                    bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                    ..Default::default()
                }]),
                ..Default::default()
            }
        );
    }
    mod bson_type_info {
        use crate::{
            json_schema::{self, simplified, BsonType, BsonTypeName},
            BsonTypeInfo,
        };

        #[test]
        fn any_of_has_two_elements_but_one_is_null_resolves_to_concrete_bson_type_info() {
            let input_schema = json_schema::Schema {
                any_of: Some(vec![
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    },
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Null)),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            };

            let input = simplified::Schema::try_from(input_schema).unwrap();

            assert_eq!(BsonTypeInfo::INT, BsonTypeInfo::from(input));
        }

        #[test]
        fn any_of_with_multiple_non_null_elements_is_not_concrete_bson_type_info() {
            let input_schema = json_schema::Schema {
                any_of: Some(vec![
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                        ..Default::default()
                    },
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::String)),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            };

            let input = simplified::Schema::try_from(input_schema).unwrap();

            assert_eq!(BsonTypeInfo::BSON, BsonTypeInfo::from(input));
        }
    }
}
