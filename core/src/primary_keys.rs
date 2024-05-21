use crate::{col_metadata::MongoColMetadata, stmt::EmptyStatement, BsonTypeInfo};
use definitions::Nullability;
use once_cell::sync::OnceCell;

static PK_METADATA: OnceCell<Vec<MongoColMetadata>> = OnceCell::new();

pub struct MongoPrimaryKeys {}

impl MongoPrimaryKeys {
    pub fn empty(max_string_length: Option<u16>) -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: PK_METADATA.get_or_init(|| {
                vec![
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "TABLE_CAT".to_string(),
                        BsonTypeInfo::STRING,
                        max_string_length,
                        Nullability::SQL_NULLABLE,
                    ),
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "TABLE_SCHEM".to_string(),
                        BsonTypeInfo::STRING,
                        max_string_length,
                        Nullability::SQL_NULLABLE,
                    ),
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "TABLE_NAME".to_string(),
                        BsonTypeInfo::STRING,
                        max_string_length,
                        Nullability::SQL_NO_NULLS,
                    ),
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "COLUMN_NAME".to_string(),
                        BsonTypeInfo::STRING,
                        max_string_length,
                        Nullability::SQL_NO_NULLS,
                    ),
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "KEY_SEQ".to_string(),
                        BsonTypeInfo::INT,
                        max_string_length,
                        Nullability::SQL_NO_NULLS,
                    ),
                    MongoColMetadata::new_metadata_from_bson_type_info_default(
                        "",
                        "".to_string(),
                        "PK_NAME".to_string(),
                        BsonTypeInfo::STRING,
                        max_string_length,
                        Nullability::SQL_NO_NULLS,
                    ),
                ]
            }),
        }
    }
}
