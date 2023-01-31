use crate::BsonTypeInfo;
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
            BsonTypeName::Undefined => BsonTypeInfo::UNDEFINED,
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
        Any,
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
                _ => Err(Error::InvalidResultSetJsonSchema),
            }
        }

        pub fn is_any(&self) -> bool {
            matches!(self, Schema::Atomic(Atomic::Any))
        }
    }

    // Converts a deserialized json_schema::Schema into a simplified::Atomic.
    // The Atomic instance is semantically equivalent to the base schema
    // when the base schema represents any valid schema other than an AnyOf.
    // To convert a possible AnyOf base schema, use the TryFrom implementation
    // for simplified::Schema.
    impl TryFrom<json_schema::Schema> for Atomic {
        type Error = Error;

        fn try_from(schema: json_schema::Schema) -> std::result::Result<Self, Self::Error> {
            match schema {
                json_schema::Schema {
                    bson_type: None,
                    properties: None,
                    required: None,
                    additional_properties: None,
                    items: None,
                    any_of: None,
                } => Ok(Atomic::Any),
                json_schema::Schema {
                    bson_type: Some(bson_type),
                    properties,
                    required,
                    additional_properties,
                    items,
                    any_of: None,
                } => {
                    let construct_obj = || {
                        Ok(Atomic::Object(ObjectSchema {
                            properties: properties
                                .unwrap_or_default()
                                .into_iter()
                                .map(|(prop, prop_schema)| {
                                    Ok((prop, Schema::try_from(prop_schema)?))
                                })
                                .collect::<Result<_>>()?,
                            required: required.unwrap_or_default().into_iter().collect(),
                            additional_properties: additional_properties.unwrap_or(true),
                        }))
                    };
                    match bson_type {
                        BsonType::Single(BsonTypeName::Array) => {
                            Ok(Atomic::Array(Box::new(match items {
                                Some(Items::Single(s)) => Schema::try_from(*s)?,
                                // The multiple-schema variant of the `items`
                                // field only asserts the schemas for the
                                // array items at specified indexes, and
                                // imposes no constraint on items at larger
                                // indexes. As such, the only schema that can
                                // describe all elements of the array is
                                // `Any`.
                                Some(Items::Multiple(_)) => Schema::Atomic(Atomic::Any),
                                None => Schema::Atomic(Atomic::Any),
                            })))
                        }
                        BsonType::Single(BsonTypeName::Object) => construct_obj(),
                        BsonType::Single(t) => Ok(Atomic::Scalar(t)),
                        // If there is a "Multiple" that contains a single type, we can simplify it.
                        BsonType::Multiple(types) if types.len() == 1 => match types.as_slice() {
                            [BsonTypeName::Object] => construct_obj(),
                            types => Ok(Atomic::Scalar(types[0])),
                        },
                        BsonType::Multiple(_) => Err(Error::InvalidResultSetJsonSchema),
                    }
                }
                _ => Err(Error::InvalidResultSetJsonSchema),
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
            match schema.clone() {
                json_schema::Schema {
                    bson_type: _bson_type,
                    properties: _properties,
                    required: _required,
                    additional_properties: _additional_properties,
                    items: _items,
                    any_of: None,
                } => Ok(Schema::Atomic(schema.try_into()?)),
                json_schema::Schema {
                    bson_type: None,
                    properties: None,
                    required: None,
                    additional_properties: None,
                    items: None,
                    any_of: Some(any_of),
                } => match any_of.len() {
                    0 => Err(Error::InvalidResultSetJsonSchema),
                    // AnyOf with a single schema is equivalent to the schema.
                    1 => Schema::try_from(any_of.into_iter().next().unwrap()),
                    _ => Ok(Schema::AnyOf(
                        any_of
                            .into_iter()
                            .map(Atomic::try_from)
                            .collect::<Result<BTreeSet<Atomic>>>()?,
                    )),
                },
                _ => Err(Error::InvalidResultSetJsonSchema),
            }
        }
    }

    impl From<Atomic> for BsonTypeInfo {
        fn from(a: Atomic) -> Self {
            match a {
                Atomic::Any => BsonTypeInfo::BSON,
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
                Schema::AnyOf(b) => (b.len() == 2)
                    .then(|| {
                        let atomics = b
                            .into_iter()
                            .filter(|a| !matches!(a, Atomic::Scalar(BsonTypeName::Null)))
                            .collect::<Vec<Atomic>>();
                        (atomics.len() == 1)
                            .then(|| atomics.first().unwrap().to_owned().into())
                            .unwrap_or(BsonTypeInfo::BSON)
                    })
                    .unwrap_or(BsonTypeInfo::BSON),
            }
        }
    }
}

#[cfg(test)]
mod unit {
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
                        Err(Error::InvalidResultSetJsonSchema),
                        Err(Error::InvalidResultSetJsonSchema),
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
            expected = Ok(Atomic::Any),
            input = json_schema::Schema::default()
        );

        try_from_test!(
            invalid_if_any_of_is_set,
            variant = Atomic,
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema),
            input = json_schema::Schema {
                any_of: Some(vec![json_schema::Schema::default()]),
                ..Default::default()
            }
        );

        try_from_test!(
            bson_type_must_not_be_none,
            variant = Atomic,
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema),
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
            expected = Err::<Atomic, Error>(Error::InvalidResultSetJsonSchema),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Multiple(vec![
                    BsonTypeName::Bool,
                    BsonTypeName::Int
                ])),
                ..Default::default()
            }
        );

        try_from_test!(
            bson_type_may_be_single_type_in_list,
            variant = Atomic,
            expected = Ok(Atomic::Scalar(BsonTypeName::Bool)),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Multiple(vec![BsonTypeName::Bool])),
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
                Atomic::Any
            )))),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                ..Default::default()
            }
        );

        try_from_test!(
            array_with_multiple_items_simplifies_to_array_of_any,
            variant = Atomic,
            expected = Ok(Atomic::Array(Box::new(simplified::Schema::Atomic(
                Atomic::Any
            )))),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Array)),
                items: Some(Items::Multiple(vec![])),
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
            expected = Ok(Schema::Atomic(Atomic::Any)),
            input = json_schema::Schema::default()
        );

        try_from_test!(
            mixing_any_of_and_other_fields_is_invalid,
            variant = Schema,
            expected = Err::<Schema, Error>(Error::InvalidResultSetJsonSchema),
            input = json_schema::Schema {
                bson_type: Some(BsonType::Single(BsonTypeName::Int)),
                any_of: Some(vec![
                    json_schema::Schema {
                        bson_type: Some(BsonType::Single(BsonTypeName::Bool)),
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
            missing_bson_type_with_other_fields_is_invalid,
            variant = Schema,
            expected = Err::<Schema, Error>(Error::InvalidResultSetJsonSchema),
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
            bson_type_info::BsonTypeInfo,
            json_schema::{self, simplified, BsonType, BsonTypeName},
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
