use crate::{
    col_metadata::MongoColMetadata,
    type_info::{BsonTypeInfo, TypeMode, SimpleBsonTypeInfo, StandardBsonTypeInfo},
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
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::Standard(StandardBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
    ];
    static ref SIMPLE_PK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::INT),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::Simple(SimpleBsonTypeInfo::STRING),
            Nullability::NO_NULLS
        ),
    ];
}

pub struct MongoPrimaryKeys {}

impl MongoPrimaryKeys {
    pub fn empty(type_mode: TypeMode) -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: match type_mode {
                TypeMode::Standard => &STANDARD_PK_METADATA,
                TypeMode::Simple => &SIMPLE_PK_METADATA,
            },
        }
    }
}
