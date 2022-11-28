use crate::{
    col_metadata::MongoColMetadata,
    conn::MongoConnection,
    err::Result,
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    stmt::MongoStatement,
    util::to_name_regex,
};
use bson::{Bson, Document};
use lazy_static::lazy_static;
use odbc_sys::Nullability;

lazy_static! {
    static ref FIELDS_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DATA_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "BUFFER_LENGTH".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DECIMAL_DIGITS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_DEF".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "CHAR_OCTET_LENGTH".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NULLABLE
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "ORDINAL_POSITION".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "IS_NULLABLE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            // the docs do not say 'not NULL', but they also say the only possible values for
            // ISO SQL are 'YES' and 'NO'. And even for non-ISO SQL they only allow additionally
            // the empty varchar... so NO_NULLS seems correct to me.
            Nullability::NO_NULLS
        ),
    ];
}

mod unit {
    #[test]
    fn metadata_size() {
        use crate::{fields::MongoFields, stmt::MongoStatement};
        assert_eq!(18, MongoFields::empty().get_resultset_metadata().len());
    }

    #[test]
    fn metadata_column_names() {
        use crate::{fields::MongoFields, stmt::MongoStatement};
        // This gives us assurance that the column names are all correct.
        assert_eq!(
            "TABLE_CAT",
            MongoFields::empty().get_col_metadata(1).unwrap().col_name
        );
        assert_eq!(
            "TABLE_SCHEM",
            MongoFields::empty().get_col_metadata(2).unwrap().col_name
        );
        assert_eq!(
            "TABLE_NAME",
            MongoFields::empty().get_col_metadata(3).unwrap().col_name
        );
        assert_eq!(
            "COLUMN_NAME",
            MongoFields::empty().get_col_metadata(4).unwrap().col_name
        );
        assert_eq!(
            "DATA_TYPE",
            MongoFields::empty().get_col_metadata(5).unwrap().col_name
        );
        assert_eq!(
            "TYPE_NAME",
            MongoFields::empty().get_col_metadata(6).unwrap().col_name
        );
        assert_eq!(
            "COLUMN_SIZE",
            MongoFields::empty().get_col_metadata(7).unwrap().col_name
        );
        assert_eq!(
            "BUFFER_LENGTH",
            MongoFields::empty().get_col_metadata(8).unwrap().col_name
        );
        assert_eq!(
            "DECIMAL_DIGITS",
            MongoFields::empty().get_col_metadata(9).unwrap().col_name
        );
        assert_eq!(
            "NUM_PREC_RADIX",
            MongoFields::empty().get_col_metadata(10).unwrap().col_name
        );
        assert_eq!(
            "NULLABLE",
            MongoFields::empty().get_col_metadata(11).unwrap().col_name
        );
        assert_eq!(
            "REMARKS",
            MongoFields::empty().get_col_metadata(12).unwrap().col_name
        );
        assert_eq!(
            "COLUMN_DEF",
            MongoFields::empty().get_col_metadata(13).unwrap().col_name
        );
        assert_eq!(
            "SQL_DATA_TYPE",
            MongoFields::empty().get_col_metadata(14).unwrap().col_name
        );
        assert_eq!(
            "SQL_DATETIME_SUB",
            MongoFields::empty().get_col_metadata(15).unwrap().col_name
        );
        assert_eq!(
            "CHAR_OCTET_LENGTH",
            MongoFields::empty().get_col_metadata(16).unwrap().col_name
        );
        assert_eq!(
            "ORDINAL_POSITION",
            MongoFields::empty().get_col_metadata(17).unwrap().col_name
        );
        assert_eq!(
            "IS_NULLABLE",
            MongoFields::empty().get_col_metadata(18).unwrap().col_name
        );
    }

    #[test]
    fn metadata_column_types() {
        use crate::{fields::MongoFields, stmt::MongoStatement};
        // This gives us assurance that the types are all correct (note
        // that we do not have smallint, so we use int, however).
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(1).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(2).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(3).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(4).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(5).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(6).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(7).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(8).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(9).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(10).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(11).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(12).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(13).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(14).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(15).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(16).unwrap().type_name
        );
        assert_eq!(
            "int",
            MongoFields::empty().get_col_metadata(17).unwrap().type_name
        );
        assert_eq!(
            "string",
            MongoFields::empty().get_col_metadata(18).unwrap().type_name
        );
    }

    #[test]
    fn metadata_column_nullability() {
        use crate::{fields::MongoFields, stmt::MongoStatement};
        use odbc_sys::Nullability;
        // This gives us assurance that the types are all correct (note
        // that we do not have smallint, so we use int, however).
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(1)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(2)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(3)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(4)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(5)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(6)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(7)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(8)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(9)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(10)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(11)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(12)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(13)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(14)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(15)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoFields::empty()
                .get_col_metadata(16)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(17)
                .unwrap()
                .nullability
        );
        // This one deviates from the docs as mentioned.
        assert_eq!(
            Nullability::NO_NULLS,
            MongoFields::empty()
                .get_col_metadata(18)
                .unwrap()
                .nullability
        );
    }
}

#[derive(Debug)]
pub struct MongoFields {
    dbs: Option<Vec<String>>,
    current_db: Option<usize>,
    collections_for_db: Option<Vec<String>>,
    current_collection: Option<usize>,
    num_fields_for_collection: Option<usize>,
    current_schema: Option<Schema>,
    current_field_for_collection: Option<usize>,
    db_name_filter: Option<Document>,
    collection_name_filter: Option<Document>,
}

// Statement related to a SQLTables call.
// The Resultset columns are hard-coded and follow the ODBC resultset for SQLColumns :
// TABLE_CAT, TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE.
impl MongoFields {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_columns(
        _mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
        db_name_filter: &str,
        collection_name_filter: &str,
    ) -> Self {
        MongoFields {
            dbs: None,
            current_db: None,
            collections_for_db: None,
            current_collection: None,
            num_fields_for_collection: None,
            current_schema: None,
            current_field_for_collection: None,
            db_name_filter: Some(to_name_regex(db_name_filter)),
            collection_name_filter: Some(to_name_regex(collection_name_filter)),
        }
    }

    pub fn empty() -> MongoFields {
        MongoFields {
            dbs: None,
            current_db: None,
            collections_for_db: None,
            current_collection: None,
            num_fields_for_collection: None,
            current_schema: None,
            current_field_for_collection: None,
            db_name_filter: None,
            collection_name_filter: None,
        }
    }
}

impl MongoStatement for MongoFields {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, _mongo_connection: Option<&MongoConnection>) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        unimplemented!()
    }

    fn get_resultset_metadata(&self) -> &Vec<crate::MongoColMetadata> {
        &FIELDS_METADATA
    }
}
