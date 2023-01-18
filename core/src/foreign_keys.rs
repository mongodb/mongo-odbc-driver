use crate::{
    col_metadata::MongoColMetadata,
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    stmt::EmptyStatement,
};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref FK_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new(
            "",
            "".to_string(),
            "PKTABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "PKTABLE_SCHEM".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "PKTABLE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "PKCOLUMN_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "FKTABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "FKTABLE_SCHEM".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "FKTABLE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "KEY_SEQ".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "UPDATE_RULE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DELETE_RULE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "FK_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "PK_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DEFERRABILITY".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
    ];
}

pub struct MongoForeignKeys {}

impl MongoForeignKeys {
    pub fn new() -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: FK_METADATA.clone(),
        }
    }
}
