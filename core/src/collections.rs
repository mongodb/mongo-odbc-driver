use crate::{
    col_metadata::MongoColMetadata,
    conn::MongoConnection,
    err::Result,
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    stmt::MongoStatement,
    util::{add_table_type_filter, to_name_regex_doc, COLLECTION, TABLE, TIMESERIES},
    Error,
};
use bson::Bson;
use lazy_static::lazy_static;
use mongodb::results::CollectionSpecification;
use mongodb::sync::Cursor;
use odbc_sys::Nullability;

lazy_static! {
    static ref COLLECTIONS_METADATA: Vec<MongoColMetadata> = vec![
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
            "TABLE_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::NULLABLE
        ),
    ];
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
    // The index of the current db.
    current_database_index: Option<usize>,
    // List of CollectionsForDb for each db.
    collections_for_db_list: Vec<CollectionsForDb>,
}

// Statement related to a SQLColumns call.
impl MongoCollections {
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_tables(
        mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
        db_name_filter: &str,
        collection_name_filter: &str,
        table_type: &str,
    ) -> Self {
        let names = mongo_connection
            .client
            .list_database_names(to_name_regex_doc(db_name_filter), None)
            .unwrap();
        let mut databases: Vec<CollectionsForDb> = Vec::with_capacity(names.len());
        for name in names {
            let list_coll_name_filter = to_name_regex_doc(collection_name_filter);
            let db = mongo_connection.client.database(name.as_str());
            let collections = db
                .list_collections(
                    add_table_type_filter(table_type, list_coll_name_filter),
                    None,
                )
                .unwrap();
            databases.push(CollectionsForDb {
                database_name: name,
                collection_list: collections,
            });
        }
        MongoCollections {
            current_collection: None,
            current_database_index: None,
            collections_for_db_list: databases,
        }
    }

    // Statement for SQLTables("", SQL_ALL_SCHEMAS,"").
    pub fn all_schemas() -> MongoCollections {
        MongoCollections::empty()
    }

    pub fn empty() -> MongoCollections {
        MongoCollections {
            current_collection: None,
            current_database_index: None,
            collections_for_db_list: Vec::new(),
        }
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next CollectionSpecification.
    // When cursor is exhausted move to next database in list
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<bool> {
        if self.current_database_index.is_none() {
            if self.collections_for_db_list.is_empty() {
                return Ok(false);
            }
            self.current_database_index = Some(0);
        }
        loop {
            if self
                .collections_for_db_list
                .get_mut(self.current_database_index.unwrap())
                .unwrap()
                .collection_list
                .advance()
                .map_err(Error::Mongo)?
            {
                // Cursor advance succeeded, update current CollectionSpecification
                self.current_collection = Some(
                    self.collections_for_db_list
                        .get(self.current_database_index.unwrap())
                        .unwrap()
                        .collection_list
                        .deserialize_current()
                        .map_err(Error::Mongo)?,
                );
                return Ok(true);
            }
            self.current_database_index = Some(self.current_database_index.unwrap() + 1);
            if self.current_database_index.unwrap() >= self.collections_for_db_list.len() {
                return Ok(false);
            }
        }
    }

    // Get the BSON value for the given colIndex on the current CollectionSpecification.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> current_collection_list.database_name
        // 2-> Schema name; NULL as it is not applicable
        // 3 -> current_collection.name
        // 4 -> current_collection.collection_type
        // 5 -> Remarks; NULL
        let return_val = match col_index {
            1 => Bson::String(
                self.collections_for_db_list
                    .get(self.current_database_index.unwrap())
                    .unwrap()
                    .database_name
                    .clone(),
            ),
            2 => Bson::Null,
            3 => Bson::String(self.current_collection.as_ref().unwrap().name.clone()),
            4 => {
                let coll_type = format!(
                    "{:?}",
                    self.current_collection.as_ref().unwrap().collection_type
                )
                .to_lowercase();
                match coll_type.as_str() {
                    // Mapping 'collection' or 'timeseries' to 'TABLE'
                    COLLECTION | TIMESERIES => Bson::String(TABLE.to_string()),
                    _ => Bson::String(coll_type.to_uppercase()),
                }
            }
            5 => Bson::String("".to_string()),
            _ => return Err(Error::ColIndexOutOfBounds(col_index)),
        };
        Ok(Some(return_val))
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &COLLECTIONS_METADATA
    }
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
        // This gives us assurance that the column names are all correct.
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
        use crate::{collections::MongoCollections, stmt::MongoStatement};
        use odbc_sys::Nullability;
        assert_eq!(
            Nullability::NO_NULLS,
            MongoCollections::empty()
                .get_col_metadata(1)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoCollections::empty()
                .get_col_metadata(2)
                .unwrap()
                .nullability
        );
        // Docs do not say NO_NULLS, but there is no way the tale name can be null.
        assert_eq!(
            Nullability::NO_NULLS,
            MongoCollections::empty()
                .get_col_metadata(3)
                .unwrap()
                .nullability
        );
        // The docs also do not say NO_NULLS, but they enumerate every possible value and
        // NULL is not one of them.
        assert_eq!(
            Nullability::NO_NULLS,
            MongoCollections::empty()
                .get_col_metadata(4)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::NULLABLE,
            MongoCollections::empty()
                .get_col_metadata(5)
                .unwrap()
                .nullability
        );
    }
    mod table_type {
        #[test]
        fn all_types() {
            use crate::collections::add_table_type_filter;
            use bson::doc;
            use constants::SQL_ALL_TABLE_TYPES;
            let filter = doc! {};
            assert_eq!(doc! {}, add_table_type_filter(SQL_ALL_TABLE_TYPES, filter));
        }
        #[test]
        fn view() {
            use crate::collections::add_table_type_filter;
            use bson::{
                doc,
                Bson::{Array, String},
            };
            let filter = doc! {};
            assert_eq!(
                doc! {"type":doc!{"$in": Array([String("view".to_string())].to_vec())}},
                add_table_type_filter("'view'", filter)
            );
        }
        #[test]
        fn table() {
            use crate::collections::add_table_type_filter;
            use bson::{
                doc,
                Bson::{Array, String},
            };
            let filter = doc! {};
            assert_eq!(
                doc! {"type":doc!{"$in": Array([String("collection".to_string()),
                String("timeseries".to_string())].to_vec())}},
                add_table_type_filter("TABLE", filter)
            );
        }
        #[test]
        fn view_table() {
            use crate::collections::add_table_type_filter;
            use bson::{
                doc,
                Bson::{Array, String},
            };
            let filter = doc! {};
            assert_eq!(
                doc! {"type":doc!{"$in": Array([String("view".to_string()),
                String("collection".to_string()),
                String("timeseries".to_string())].to_vec())}},
                add_table_type_filter("View,'table'", filter)
            );
        }
        #[test]
        fn not_supported() {
            use crate::collections::add_table_type_filter;
            use bson::{doc, Bson::Array};
            let filter = doc! {};
            assert_eq!(
                doc! {"type":doc!{"$in": Array([].to_vec())}},
                add_table_type_filter("GLOBAL TEMPORARY", filter)
            );
        }
    }
}
