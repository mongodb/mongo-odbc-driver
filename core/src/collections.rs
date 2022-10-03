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
use bson::Bson;
use lazy_static::lazy_static;
use mongodb::results::CollectionSpecification;
use mongodb::sync::Cursor;

lazy_static! {
    static ref COLLECTIONS_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::NoNulls
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
            "TABLE_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::NoNulls
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
        ),
    ];
}

mod unit {
    #[test]
    fn metadata_size() {
        use crate::{collections::MongoCollections, stmt::MongoStatement};
        assert_eq!(5, MongoCollections::empty().get_resultset_metadata().len());
    }

    #[test]
    fn metadata_column_names() {
        use crate::{collections::MongoCollections, stmt::MongoStatement};
        // These were generated straight from the docs (hence the - 1). This
        // gives us assurance that the column names are all correct.
        assert_eq!(
            "TABLE_CAT",
            MongoCollections::empty()
                .get_col_metadata(1)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_SCHEM",
            MongoCollections::empty()
                .get_col_metadata(2)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_NAME",
            MongoCollections::empty()
                .get_col_metadata(3)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_TYPE",
            MongoCollections::empty()
                .get_col_metadata(4)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "REMARKS",
            MongoCollections::empty()
                .get_col_metadata(5)
                .unwrap()
                .col_name
        );
    }

    #[test]
    fn metadata_column_types() {
        use crate::{collections::MongoCollections, stmt::MongoStatement};
        // These were generated straight from the docs (hence the - 1).
        assert_eq!(
            "string",
            MongoCollections::empty()
                .get_col_metadata(1)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoCollections::empty()
                .get_col_metadata(2)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoCollections::empty()
                .get_col_metadata(3)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoCollections::empty()
                .get_col_metadata(4)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoCollections::empty()
                .get_col_metadata(5)
                .unwrap()
                .type_name
        );
    }

    #[test]
    fn metadata_column_nullability() {
        use crate::col_metadata::ColumnNullability;
        use crate::{collections::MongoCollections, stmt::MongoStatement};
        // These were generated straight from the docs (hence the - 1).
        assert_eq!(
            ColumnNullability::NoNulls,
            MongoCollections::empty()
                .get_col_metadata(1)
                .unwrap()
                .is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            MongoCollections::empty()
                .get_col_metadata(2)
                .unwrap()
                .is_nullable
        );
        // Docs do not say NoNulls, but there is no way the tale name can be null.
        assert_eq!(
            ColumnNullability::NoNulls,
            MongoCollections::empty()
                .get_col_metadata(3)
                .unwrap()
                .is_nullable
        );
        // The docs also do not say NoNulls, but they enumerate every possible value and
        // NULL is not one of them.
        assert_eq!(
            ColumnNullability::NoNulls,
            MongoCollections::empty()
                .get_col_metadata(4)
                .unwrap()
                .is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            MongoCollections::empty()
                .get_col_metadata(5)
                .unwrap()
                .is_nullable
        );
    }
}

#[derive(Debug)]
struct CollectionsForDb {
    database_name: String,
    collection_list: Cursor<CollectionSpecification>,
}

#[derive(Debug)]
pub struct MongoCollections {
    // The current collection specification.
    current_collection: Option<CollectionSpecification>,
    // The cursor on the collections specification for the db.
    current_collection_list: Option<CollectionsForDb>,
}

// Statement related to a SQLColumns call.
impl MongoCollections {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_tables(
        _client: &MongoConnection,
        _query_timeout: Option<i32>,
        _db_name_filter: &str,
        _collection_name_filter: &str,
    ) -> Self {
        unimplemented!()
    }

    fn empty() -> MongoCollections {
        MongoCollections {
            current_collection: None,
            current_collection_list: None,
        }
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next CollectionSpecification.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        unimplemented!()
    }

    // Get the BSON value for the given colIndex on the current CollectionSpecification.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> current_coll_list.0
        // 2 -> current_coll.name
        // 3 -> current_coll.CollectionType
        unimplemented!()
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &*COLLECTIONS_METADATA
    }
}
