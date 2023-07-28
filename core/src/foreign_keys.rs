use crate::{
    col_metadata::MongoColMetadata,
    stmt::EmptyStatement,
    BsonTypeInfo, TypeMode,

};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref FK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKCOLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKCOLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            BsonTypeInfo::INT,
            TypeMode::Standard,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "UPDATE_RULE".to_string(),
            BsonTypeInfo::INT,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DELETE_RULE".to_string(),
            BsonTypeInfo::INT,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FK_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            BsonTypeInfo::STRING,
            TypeMode::Standard,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DEFERRABILITY".to_string(),
            BsonTypeInfo::INT,
            TypeMode::Standard,
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
