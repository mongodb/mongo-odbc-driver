use crate::{col_metadata::MongoColMetadata, stmt::EmptyStatement, BsonTypeInfo};
use definitions::Nullability;
use lazy_static::lazy_static;

lazy_static! {
    static ref PK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
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
