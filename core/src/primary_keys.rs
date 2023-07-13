use crate::{
    bson_type_info::{BsonTypeInfo, SchemaMode, SimpleTypeInfo, StandardTypeInfo},
    col_metadata::MongoColMetadata,
    stmt::EmptyStatement,
};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref STANDARD_PK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::Standard(StandardTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
    ];
    static ref SIMPLE_PK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
    ];
}

pub struct MongoPrimaryKeys {}

impl MongoPrimaryKeys {
    pub fn empty(schema_mode: SchemaMode) -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: match schema_mode {
                SchemaMode::Standard => &STANDARD_PK_METADATA,
                SchemaMode::Simple => &SIMPLE_PK_METADATA,
            },
        }
    }
}
