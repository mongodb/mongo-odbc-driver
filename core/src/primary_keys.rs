use crate::{bson_type_info::BsonTypeInfo, col_metadata::MongoColMetadata, stmt::EmptyStatement};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref PK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::INT,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
    ];
}

pub struct MongoPrimaryKeys {}

impl MongoPrimaryKeys {
    pub fn empty() -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: &PK_METADATA,
        }
    }
}
