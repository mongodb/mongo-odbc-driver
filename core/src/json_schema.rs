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
        /// A datasource schema must be an Object schema. Unlike Object schemata
        /// in general, the properties field cannot be null.
        pub fn assert_datasource_schema(&self) -> Result<&ObjectSchema> {
            match self {
                Schema::Atomic(Atomic::Object(s)) => Ok(s),
                _ => Err(Error::InvalidResultSetJsonSchema),
            }
        }

        pub fn is_any(&self) -> bool {
            match self {
                Schema::Atomic(Atomic::Any) => true,
                _ => false,
            }
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
                } => match bson_type {
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
                    BsonType::Multiple(_) => Err(Error::InvalidResultSetJsonSchema),
                },
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
                    bson_type: None,
                    properties: None,
                    required: None,
                    additional_properties: None,
                    items: None,
                    any_of: None,
                } => Ok(Schema::Atomic(Atomic::Any)),
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
                } => Ok(Schema::AnyOf(
                    any_of
                        .into_iter()
                        .map(Atomic::try_from)
                        .collect::<Result<BTreeSet<Atomic>>>()?,
                )),
                _ => Err(Error::InvalidResultSetJsonSchema),
            }
        }
    }

    impl From<Schema> for BsonTypeInfo {
        fn from(v: Schema) -> Self {
            match v {
                Schema::Atomic(Atomic::Any) => BsonTypeInfo::BSON,
                Schema::Atomic(Atomic::Scalar(t)) => t.into(),
                Schema::Atomic(Atomic::Object(_)) => BsonTypeInfo::OBJECT,
                Schema::Atomic(Atomic::Array(_)) => BsonTypeInfo::ARRAY,
                Schema::AnyOf(_) => BsonTypeInfo::BSON,
            }
        }
    }
}
