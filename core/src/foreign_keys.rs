use crate::{
    bson_type_info::standard_type_info::StandardTypeInfo, col_metadata::MongoColMetadata,
    stmt::EmptyStatement,
};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref FK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_CAT".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_SCHEM".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKTABLE_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PKCOLUMN_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_CAT".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_SCHEM".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKTABLE_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FKCOLUMN_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            StandardTypeInfo::INT,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "UPDATE_RULE".to_string(),
            StandardTypeInfo::INT,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DELETE_RULE".to_string(),
            StandardTypeInfo::INT,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "FK_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            StandardTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info(
            "",
            "".to_string(),
            "DEFERRABILITY".to_string(),
            StandardTypeInfo::INT,
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
