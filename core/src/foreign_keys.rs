use crate::{col_metadata::MongoColMetadata, stmt::EmptyStatement, BsonTypeInfo};
use lazy_static::lazy_static;
use definitions::Nullability;

lazy_static! {
    static ref FK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PKTABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PKTABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PKTABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PKCOLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FKTABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FKTABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FKTABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FKCOLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::INT,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "UPDATE_RULE".to_string(),
            BsonTypeInfo::INT,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "DELETE_RULE".to_string(),
            BsonTypeInfo::INT,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "FK_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "DEFERRABILITY".to_string(),
            BsonTypeInfo::INT,
            Nullability::NULLABLE
        ),
    ];
}

pub struct MongoForeignKeys {}

impl MongoForeignKeys {
    pub fn empty() -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: &FK_METADATA,
        }
    }
}
