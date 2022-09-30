use crate::{
    col_metadata::{ColumnNullability, MongoColMetadata},
    conn::MongoConnection,
    err::Result,
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    stmt::MongoStatement,
};
use bson::{Bson, Document};
use lazy_static::lazy_static;
use mongodb::sync::Cursor;

lazy_static! {
    static ref FIELDS_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DATA_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "BUFFER_LENGTH".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "DECIMAL_DIGITS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "COLUMN_DEF".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "CHAR_OCTET_LENGTH".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "ORDINAL_POSITION".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "IS_NULLABLE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            // the docs do not say 'not NULL', but they also say the only possible values for
            // ISO SQL are 'YES' and 'NO'. And even for non-ISO SQL they only allow additionally
            // the empty varchar... so NoNulls seems correct to me.
            ColumnNullability::NoNulls
        ),
    ];
}

mod unit {
    #[test]
    fn metadata_size() {
        assert_eq!(18, super::FIELDS_METADATA.len());
    }

    #[test]
    fn metadata_column_names() {
        // These were generated straight from the docs (hence the - 1). This
        // gives us assurance that the column names are all correct.
        assert_eq!("TABLE_CAT", super::FIELDS_METADATA[1 - 1].col_name);
        assert_eq!("TABLE_SCHEM", super::FIELDS_METADATA[2 - 1].col_name);
        assert_eq!("TABLE_NAME", super::FIELDS_METADATA[3 - 1].col_name);
        assert_eq!("COLUMN_NAME", super::FIELDS_METADATA[4 - 1].col_name);
        assert_eq!("DATA_TYPE", super::FIELDS_METADATA[5 - 1].col_name);
        assert_eq!("TYPE_NAME", super::FIELDS_METADATA[6 - 1].col_name);
        assert_eq!("COLUMN_SIZE", super::FIELDS_METADATA[7 - 1].col_name);
        assert_eq!("BUFFER_LENGTH", super::FIELDS_METADATA[8 - 1].col_name);
        assert_eq!("DECIMAL_DIGITS", super::FIELDS_METADATA[9 - 1].col_name);
        assert_eq!("NUM_PREC_RADIX", super::FIELDS_METADATA[10 - 1].col_name);
        assert_eq!("NULLABLE", super::FIELDS_METADATA[11 - 1].col_name);
        assert_eq!("REMARKS", super::FIELDS_METADATA[12 - 1].col_name);
        assert_eq!("COLUMN_DEF", super::FIELDS_METADATA[13 - 1].col_name);
        assert_eq!("SQL_DATA_TYPE", super::FIELDS_METADATA[14 - 1].col_name);
        assert_eq!("SQL_DATETIME_SUB", super::FIELDS_METADATA[15 - 1].col_name);
        assert_eq!("CHAR_OCTET_LENGTH", super::FIELDS_METADATA[16 - 1].col_name);
        assert_eq!("ORDINAL_POSITION", super::FIELDS_METADATA[17 - 1].col_name);
        assert_eq!("IS_NULLABLE", super::FIELDS_METADATA[18 - 1].col_name);
    }

    #[test]
    fn metadata_column_types() {
        // These were generated straight from the docs (hence the - 1). This
        // gives us assurance that the types are all correct (note that we do not have smallint, so
        // we use int, however).
        assert_eq!("string", super::FIELDS_METADATA[1 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[2 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[3 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[4 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[5 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[6 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[7 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[8 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[9 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[10 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[11 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[12 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[13 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[14 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[15 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[16 - 1].type_name);
        assert_eq!("int", super::FIELDS_METADATA[17 - 1].type_name);
        assert_eq!("string", super::FIELDS_METADATA[18 - 1].type_name);
    }

    fn metadata_column_nullability() {
        use crate::col_metadata::ColumnNullability;
        // These were generated straight from the docs (hence the - 1). This
        // gives us assurance that the types are all correct (note that we do not have smallint, so
        // we use int, however).
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[1 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[2 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[3 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[4 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[5 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[6 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[7 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[8 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[9 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[10 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[11 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[12 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[13 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[14 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[15 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::FIELDS_METADATA[16 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[17 - 1].is_nullable
        );
        // This one deviates from the docs as mentioned.
        assert_eq!(
            ColumnNullability::NoNulls,
            super::FIELDS_METADATA[18 - 1].is_nullable
        );
    }
}

#[derive(Debug)]
struct FieldsForCollection {
    database_name: String,
    collection_name: String,
    // Info retrieved via sqlgetschema
    // See https://docs.mongodb.com/datalake/reference/cli/sql/sqlgetschema/ for more details.
    schema: Cursor<Document>,
}

#[derive(Debug)]
pub struct MongoFields {
    // The current collection specification.
    current_field_list: Option<FieldsForCollection>,
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
        _client: &MongoConnection,
        _query_timeout: Option<i32>,
        _db_name_filter: &str,
        _collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }
}

impl MongoStatement for MongoFields {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        unimplemented!()
    }

    fn get_resultset_metadata(&self) -> &Vec<crate::MongoColMetadata> {
        &*FIELDS_METADATA
    }
}
