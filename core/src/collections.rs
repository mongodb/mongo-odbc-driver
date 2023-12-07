use crate::stmt::EmptyStatement;
use crate::util::{is_match, table_type_filter_to_vec, to_name_regex};
use crate::{
    col_metadata::MongoColMetadata,
    conn::MongoConnection,
    err::Result,
    stmt::MongoStatement,
    util::{COLLECTION, TABLE, TIMESERIES},
    BsonTypeInfo, Error,
};
use bson::{doc, Bson};
use lazy_static::lazy_static;
use mongodb::{options::ListDatabasesOptions, results::CollectionType};
use odbc_sys::Nullability;
use regex::Regex;

lazy_static! {
    static ref COLLECTIONS_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_TYPE".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            BsonTypeInfo::STRING,
            Nullability::NULLABLE
        ),
    ];
}

#[derive(Debug, Clone)]
pub(crate) struct MongoODBCCollectionSpecification {
    pub name: String,
    pub collection_type: CollectionType,
}

impl MongoODBCCollectionSpecification {
    pub fn new(name: String, collection_type: CollectionType) -> Self {
        Self {
            name,
            collection_type,
        }
    }
}

#[derive(Debug)]
struct CollectionsForDb {
    database_name: String,
    collection_list: Vec<MongoODBCCollectionSpecification>,
}

#[derive(Debug)]
pub struct MongoCollections {
    // The current collection specification.
    current_collection: Option<MongoODBCCollectionSpecification>,
    // The index of the current collection.
    current_collection_index: Option<usize>,
    // The index of the current db.
    current_database_index: Option<usize>,
    // List of CollectionsForDb for each db.
    collections_for_db_list: Vec<CollectionsForDb>,
    collection_name_filter: Option<Regex>,
    table_types_filter: Option<Vec<CollectionType>>,
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
        accept_search_patterns: bool,
    ) -> Self {
        let databases = mongo_connection
            .client
            .list_database_names(
                None,
                ListDatabasesOptions::builder()
                    .authorized_databases(true)
                    .build(),
            )
            .unwrap()
            .iter()
            // MHOUSE-7119 - admin database and empty strings are showing in list_database_names
            .filter(|&db_name| !db_name.is_empty() && !db_name.eq("admin"))
            .filter(|&db_name| is_match(db_name, db_name_filter, accept_search_patterns))
            .map(|val| {
                CollectionsForDb {
                database_name: val.to_string(),
                collection_list: mongo_connection.client.database(val.as_str()).run_command(
                    doc! { "listCollections": 1, "nameOnly": true, "authorizedCollections": true},
                    None,
                ).unwrap().get_document("cursor").map(|doc| {
                    doc.get_array("firstBatch").unwrap().iter().map(|val| {
                        let doc = val.as_document().unwrap();
                        let name = doc.get_str("name").unwrap().to_string();
                        let collection_type = match doc.get_str("type").unwrap() {
                            "collection" => CollectionType::Collection,
                            "view" => CollectionType::View,
                            _ => CollectionType::Collection
                        };
                        MongoODBCCollectionSpecification::new(name, collection_type)
                    }).collect()
                }).unwrap_or_else(|_| {
                    log::error!("Error getting collections for db {}", val);
                    vec![]
                }),
            }
            })
            .collect();

        MongoCollections {
            current_collection: None,
            current_collection_index: None,
            current_database_index: None,
            collections_for_db_list: databases,
            collection_name_filter: to_name_regex(collection_name_filter),
            table_types_filter: table_type_filter_to_vec(table_type),
        }
    }

    // Statement for SQLTables("", SQL_ALL_SCHEMAS,"").
    pub fn all_schemas() -> EmptyStatement {
        EmptyStatement {
            resultset_metadata: &COLLECTIONS_METADATA,
        }
    }

    pub fn empty() -> MongoCollections {
        MongoCollections {
            current_collection: None,
            current_collection_index: None,
            current_database_index: None,
            collections_for_db_list: Vec::new(),
            table_types_filter: None,
            collection_name_filter: None,
        }
    }
}

impl MongoStatement for MongoCollections {
    // Move the cursor to the next CollectionSpecification.
    // When cursor is exhausted move to next database in list
    // Return true if moving was successful, false otherwise.
    #[allow(clippy::blocks_in_if_conditions)]
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        if self.current_database_index.is_none() {
            if self.collections_for_db_list.is_empty() {
                return Ok((false, vec![]));
            }
            self.current_database_index = Some(0);
        }
        if self.current_collection_index.is_none() {
            self.current_collection_index = Some(0);
        }

        loop {
            while self
                .collections_for_db_list
                .get_mut(self.current_database_index.unwrap())
                .unwrap()
                .collection_list
                .get_mut(self.current_collection_index.unwrap())
                .is_some()
            {
                match self
                    .collections_for_db_list
                    .get(self.current_database_index.unwrap())
                    .unwrap()
                    .collection_list
                    .get(self.current_collection_index.unwrap())
                {
                    Some(collection) => {
                        if (self.table_types_filter.is_none()
                            || self
                                .table_types_filter
                                .as_ref()
                                .unwrap()
                                .contains(&collection.collection_type))
                            && (self.collection_name_filter.is_none()
                                || self
                                    .collection_name_filter
                                    .as_ref()
                                    .unwrap()
                                    .is_match(&collection.name))
                        {
                            // collection matches the filters, advance the collection index and update current CollectionSpecification
                            self.current_collection_index =
                                self.current_collection_index.map(|val| val + 1);
                            self.current_collection = Some(collection.clone());
                            return Ok((true, vec![]));
                        } else {
                            // collection doens't match the filter, advance the collection index
                            self.current_collection_index =
                                self.current_collection_index.map(|val| val + 1);
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }

            self.current_database_index = Some(self.current_database_index.unwrap() + 1);
            self.current_collection_index = Some(0);
            if self.current_database_index.unwrap() >= self.collections_for_db_list.len() {
                return Ok((false, vec![]));
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

    #[cfg(test)]
    mod table_type {
        use crate::util::table_type_filter_to_vec;
        use constants::SQL_ALL_TABLE_TYPES;
        use mongodb::results::CollectionType;

        #[test]
        fn all_types() {
            let filters_opt = table_type_filter_to_vec(SQL_ALL_TABLE_TYPES);
            // No filtering will be required
            assert!(filters_opt.is_none());
        }
        #[test]
        fn view() {
            let filters_opt = table_type_filter_to_vec("View");
            assert!(filters_opt.is_some());
            let filters = filters_opt.unwrap();
            assert_eq!(filters.len(), 1);
            assert!(filters.contains(&CollectionType::View));
        }
        #[test]
        fn table() {
            let filters_opt = table_type_filter_to_vec("table");
            assert!(filters_opt.is_some());
            let filters = filters_opt.unwrap();
            assert_eq!(filters.len(), 1);
            assert!(filters.contains(&CollectionType::Collection));
        }
        #[test]
        fn view_table() {
            let filters_opt = table_type_filter_to_vec("view,'table'");
            assert!(filters_opt.is_some());
            let filters = filters_opt.unwrap();
            assert_eq!(filters.len(), 2);
            assert!(filters.contains(&CollectionType::Collection));
            assert!(filters.contains(&CollectionType::View));
        }
        #[test]
        fn some_not_supported() {
            let filters_opt = table_type_filter_to_vec("TABLE, GLOBAL TEMPORARY");
            assert!(filters_opt.is_some());
            let filters = filters_opt.unwrap();
            assert_eq!(filters.len(), 1);
            assert!(filters.contains(&CollectionType::Collection));
        }
        #[test]
        fn none_supported() {
            let filters_opt = table_type_filter_to_vec("GLOBAL TEMPORARY, SYSTEM TABLE");
            assert!(filters_opt.is_some());
            let filters = filters_opt.unwrap();
            assert!(filters.is_empty());
        }
    }
}
