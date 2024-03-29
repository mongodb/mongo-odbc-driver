use crate::{
    col_metadata::{MongoColMetadata, SqlGetSchemaResponse},
    collections::MongoODBCCollectionSpecification,
    conn::MongoConnection,
    err::{Error, Result},
    stmt::MongoStatement,
    util::to_name_regex,
    BsonTypeInfo, TypeMode,
};
use bson::{doc, Bson};
use definitions::{Nullability, SqlDataType};
use lazy_static::lazy_static;
use mongodb::{options::ListDatabasesOptions, results::CollectionType};
use regex::Regex;
use std::collections::VecDeque;

lazy_static! {
    static ref FIELDS_METADATA: Vec<MongoColMetadata> = vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "COLUMN_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "DATA_TYPE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TYPE_NAME".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "COLUMN_SIZE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "BUFFER_LENGTH".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "DECIMAL_DIGITS".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "NUM_PREC_RADIX".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "NULLABLE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "COLUMN_DEF".to_string(),
            BsonTypeInfo::STRING,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "SQL_DATA_TYPE".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "SQL_DATETIME_SUB".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "CHAR_OCTET_LENGTH".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NULLABLE
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "ORDINAL_POSITION".to_string(),
            BsonTypeInfo::INT,
            Nullability::SQL_NO_NULLS
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "IS_NULLABLE".to_string(),
            BsonTypeInfo::STRING,
            // the docs do not say 'not NULL', but they also say the only possible values for
            // ISO SQL are 'YES' and 'NO'. And even for non-ISO SQL they only allow additionally
            // the empty varchar... so NO_NULLS seems correct to me.
            Nullability::SQL_NO_NULLS
        ),
    ];
}

// mod unit {
//     #[test]
//     fn metadata_size() {
//         use crate::{fields::MongoFields, stmt::MongoStatement};
//         assert_eq!(18, MongoFields::empty().get_resultset_metadata().len());
//     }

//     #[test]
//     fn metadata_column_names() {
//         use crate::{fields::MongoFields, stmt::MongoStatement};
//         // This gives us assurance that the column names are all correct.
//         assert_eq!(
//             "TABLE_CAT",
//             MongoFields::empty().get_col_metadata(1).unwrap().col_name
//         );
//         assert_eq!(
//             "TABLE_SCHEM",
//             MongoFields::empty().get_col_metadata(2).unwrap().col_name
//         );
//         assert_eq!(
//             "TABLE_NAME",
//             MongoFields::empty().get_col_metadata(3).unwrap().col_name
//         );
//         assert_eq!(
//             "COLUMN_NAME",
//             MongoFields::empty().get_col_metadata(4).unwrap().col_name
//         );
//         assert_eq!(
//             "DATA_TYPE",
//             MongoFields::empty().get_col_metadata(5).unwrap().col_name
//         );
//         assert_eq!(
//             "TYPE_NAME",
//             MongoFields::empty().get_col_metadata(6).unwrap().col_name
//         );
//         assert_eq!(
//             "COLUMN_SIZE",
//             MongoFields::empty().get_col_metadata(7).unwrap().col_name
//         );
//         assert_eq!(
//             "BUFFER_LENGTH",
//             MongoFields::empty().get_col_metadata(8).unwrap().col_name
//         );
//         assert_eq!(
//             "DECIMAL_DIGITS",
//             MongoFields::empty().get_col_metadata(9).unwrap().col_name
//         );
//         assert_eq!(
//             "NUM_PREC_RADIX",
//             MongoFields::empty().get_col_metadata(10).unwrap().col_name
//         );
//         assert_eq!(
//             "NULLABLE",
//             MongoFields::empty().get_col_metadata(11).unwrap().col_name
//         );
//         assert_eq!(
//             "REMARKS",
//             MongoFields::empty().get_col_metadata(12).unwrap().col_name
//         );
//         assert_eq!(
//             "COLUMN_DEF",
//             MongoFields::empty().get_col_metadata(13).unwrap().col_name
//         );
//         assert_eq!(
//             "SQL_DATA_TYPE",
//             MongoFields::empty().get_col_metadata(14).unwrap().col_name
//         );
//         assert_eq!(
//             "SQL_DATETIME_SUB",
//             MongoFields::empty().get_col_metadata(15).unwrap().col_name
//         );
//         assert_eq!(
//             "CHAR_OCTET_LENGTH",
//             MongoFields::empty().get_col_metadata(16).unwrap().col_name
//         );
//         assert_eq!(
//             "ORDINAL_POSITION",
//             MongoFields::empty().get_col_metadata(17).unwrap().col_name
//         );
//         assert_eq!(
//             "IS_NULLABLE",
//             MongoFields::empty().get_col_metadata(18).unwrap().col_name
//         );
//     }

//     #[test]
//     fn metadata_column_types() {
//         use crate::{fields::MongoFields, stmt::MongoStatement};
//         // This gives us assurance that the types are all correct (note
//         // that we do not have smallint, so we use int, however).
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(1).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(2).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(3).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(4).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(5).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(6).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(7).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(8).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(9).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(10).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(11).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(12).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(13).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(14).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(15).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(16).unwrap().type_name
//         );
//         assert_eq!(
//             "int",
//             MongoFields::empty().get_col_metadata(17).unwrap().type_name
//         );
//         assert_eq!(
//             "string",
//             MongoFields::empty().get_col_metadata(18).unwrap().type_name
//         );
//     }

//     #[test]
//     fn metadata_column_nullability() {
//         use crate::{fields::MongoFields, stmt::MongoStatement};
//         use definitions::Nullability;
//         // This gives us assurance that the types are all correct (note
//         // that we do not have smallint, so we use int, however).
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(1)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(2)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(3)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(4)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(5)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(6)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(7)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(8)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(9)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(10)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(11)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(12)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(13)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(14)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(15)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NULLABLE,
//             MongoFields::empty()
//                 .get_col_metadata(16)
//                 .unwrap()
//                 .nullability
//         );
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(17)
//                 .unwrap()
//                 .nullability
//         );
//         // This one deviates from the docs as mentioned.
//         assert_eq!(
//             Nullability::SQL_NO_NULLS,
//             MongoFields::empty()
//                 .get_col_metadata(18)
//                 .unwrap()
//                 .nullability
//         );
//     }
// }

#[derive(Debug)]
pub struct MongoFields {
    dbs: VecDeque<String>,
    current_db_name: String,
    collections_for_db: Option<VecDeque<MongoODBCCollectionSpecification>>,
    current_col_metadata: Vec<MongoColMetadata>,
    current_field_for_collection: isize,
    collection_name_filter: Option<Regex>,
    field_name_filter: Option<Regex>,
    type_mode: TypeMode,
    /// Whether this mongofield should map to odbc 3 types or not
    odbc_3_types: bool,
}

// Statement related to a SQLTables call.
// The Resultset columns are hard-coded and follow the ODBC resultset for SQLColumns :
// TABLE_CAT, TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE.
impl MongoFields {
    /// Whether to map the TIMESTAMP type (93) to EXT_TIMESTAMP (11) for odbc 2. Maps the type if `odbc_3_types` is `false`
    /// AND the data_type is SqlDataType::TIMESTAMP, otherwise, this is an identity function.
    /// See https://learn.microsoft.com/en-us/sql/odbc/reference/develop-app/datetime-data-type-changes?view=sql-server-ver16 for more information.
    pub fn map_type_for_odbc_version(odbc_3_types: bool, data_type: SqlDataType) -> SqlDataType {
        match (odbc_3_types, data_type) {
            (false, SqlDataType::SQL_TYPE_TIMESTAMP) => SqlDataType::SQL_TIMESTAMP,
            _ => data_type,
        }
    }
    // Create a new MongoStatement to list tables with the given database (catalogs) and collection
    // (tables) names filters.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_columns(
        mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
        db_name: Option<&str>,
        collection_name_filter: Option<&str>,
        field_name_filter: Option<&str>,
        type_mode: TypeMode,
        odbc_3_types: bool,
    ) -> Self {
        let dbs = db_name.map_or_else(
            || {
                let _guard = mongo_connection.runtime.enter();
                mongo_connection.runtime.block_on(async {
                    mongo_connection
                        .client
                        .list_database_names(
                            None,
                            ListDatabasesOptions::builder()
                                .authorized_databases(true)
                                .build(),
                        )
                        .await
                        .unwrap()
                        // MHOUSE-7119 - admin database and empty strings are showing in list_database_names
                        .iter()
                        .filter(|&db_name| !db_name.is_empty() && !db_name.eq("admin"))
                        .map(|s| s.to_string())
                        .collect()
                })
            },
            |db| vec![db.to_string()],
        );
        MongoFields {
            dbs: dbs.into(),
            current_db_name: "".to_string(),
            collections_for_db: None,
            current_col_metadata: Vec::new(),
            current_field_for_collection: -1,
            collection_name_filter: collection_name_filter.and_then(to_name_regex),
            field_name_filter: field_name_filter.and_then(to_name_regex),
            type_mode,
            odbc_3_types,
        }
    }

    pub fn empty() -> MongoFields {
        MongoFields {
            dbs: VecDeque::new(),
            current_db_name: "".to_string(),
            collections_for_db: None,
            current_col_metadata: Vec::new(),
            current_field_for_collection: -1,
            collection_name_filter: None,
            field_name_filter: None,
            type_mode: TypeMode::Standard,
            odbc_3_types: true,
        }
    }

    fn get_next_metadata(
        &mut self,
        mongo_connection: &MongoConnection,
    ) -> Result<(bool, Vec<Error>)> {
        let _guard = mongo_connection.runtime.enter();
        mongo_connection.runtime.block_on(async {
            let mut warnings: Vec<Error> = vec![];
            loop {
                if self.collections_for_db.is_some() {
                    if let Some(current_collection) =
                        self.collections_for_db.as_mut().unwrap().pop_front()
                    {
                        let collection_name = current_collection.name.clone();
                        if self.collection_name_filter.is_some()
                            && !self
                                .collection_name_filter
                                .as_ref()
                                .unwrap()
                                .is_match(&collection_name)
                        {
                            // The collection does not match the filter, moving to the next one
                            continue;
                        }
                        let get_schema_cmd = doc! {"sqlGetSchema": collection_name.clone()};

                        let db = mongo_connection.client.database(&self.current_db_name);
                        let current_col_metadata_response: Result<SqlGetSchemaResponse> =
                            bson::from_document(
                                db.run_command(get_schema_cmd, None).await.unwrap(),
                            )
                            .map_err(|e| {
                                Error::CollectionDeserialization(collection_name.clone(), e)
                            });
                        if let Err(error) = current_col_metadata_response {
                            // If there is an Error while deserializing the schema, we won't show any columns for it
                            warnings.push(error);
                            continue;
                        }
                        let current_col_metadata_response = current_col_metadata_response.unwrap();
                        match current_col_metadata_response.process_collection_metadata(
                            &self.current_db_name,
                            collection_name.as_str(),
                            self.type_mode,
                        ) {
                            Ok(current_col_metadata) => {
                                if !current_col_metadata.is_empty() {
                                    self.current_col_metadata = current_col_metadata;
                                    self.current_field_for_collection = 0;
                                    return Ok((true, warnings));
                                }
                            }
                            // If there is an error simplifying the schema (e.g. an AnyOf), skip the collection
                            Err(e) => {
                                log::error!("Error while processing collection metadata: {}", e);
                                continue;
                            }
                        }
                    }
                }
                if self.dbs.is_empty() {
                    return Ok((false, warnings));
                }
                let db_name = self.dbs.pop_front().unwrap();
                self.collections_for_db = Some(
                mongo_connection
                    .client
                    .database(&db_name)
                    .run_command(
                    doc! { "listCollections": 1, "nameOnly": true, "authorizedCollections": true},
                    None,
                ).await.unwrap().get_document("cursor").map(|doc| {
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
                    log::error!("Error getting collections for database {db_name}");
                    VecDeque::new()
                }),
            );
                self.current_db_name = db_name;
            }
        })
    }
}

impl MongoStatement for MongoFields {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, mongo_connection: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        match self.field_name_filter.as_ref() {
            None => {
                self.current_field_for_collection += 1;
                match (self.current_field_for_collection as usize) < self.current_col_metadata.len()
                {
                    true => Ok((true, vec![])),
                    false => self.get_next_metadata(mongo_connection.unwrap()),
                }
            }
            Some(filter) => {
                let filter = filter.clone();
                let mut warnings: Vec<Error> = vec![];
                loop {
                    self.current_field_for_collection += 1;
                    let parse_warnings = |res: (bool, Vec<Error>)| {
                        warnings.extend(res.1);
                        res.0
                    };
                    if (self.current_field_for_collection as usize
                        >= self.current_col_metadata.len())
                        && !self
                            .get_next_metadata(mongo_connection.unwrap())
                            .map(parse_warnings)
                            .unwrap()
                    {
                        return Ok((false, warnings));
                    }
                    if filter.is_match(
                        &self
                            .current_col_metadata
                            .get(self.current_field_for_collection as usize)
                            .unwrap()
                            .col_name,
                    ) {
                        return Ok((true, warnings));
                    }
                }
            }
        }
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // use self.current_col_metadata[current_field_for_collection]
        // 1 -> TABLE_CAT
        // 2 -> TABLE_SCHEM  (NULL)
        // 3 -> TABLE_NAME
        // 4 -> COLUMN_NAME
        // 5 -> DATA_TYPE
        // 6 -> TYPE_NAME
        // 7 -> COLUMN_SIZE
        // 8 -> BUFFER_LENGTH
        // 9 -> DECIMAL_DIGITS
        // 10 -> NUM_PREC_RADIX
        // 11 -> NULLABLE
        // 12 -> REMARKS
        // 13 -> COLUMN_DEF
        // 14 -> SQL_DATA_TYPE
        // 15 -> SQL_DATETIME_SUB
        // 16 -> CHAR_OCTET_LENGTH
        // 17 -> ORDINAL_POSITION
        // 18 -> IS_NULLABLE "YES" or "NO"
        let get_meta_data = || {
            self.current_col_metadata
                .get(self.current_field_for_collection as usize)
                .ok_or(Error::InvalidCursorState)
        };
        Ok(Some(match col_index {
            1 => Bson::String(self.current_db_name.clone()),
            2 => Bson::Null,
            3 => Bson::String(get_meta_data()?.table_name.clone()),
            4 => Bson::String(get_meta_data()?.col_name.clone()),
            5 => Bson::Int32(MongoFields::map_type_for_odbc_version(
                !self.odbc_3_types,
                get_meta_data()?.sql_type,
            ) as i32),
            6 => Bson::String(get_meta_data()?.type_name.clone()),
            7 => Bson::Int32(get_meta_data()?.precision.unwrap_or(0) as i32),
            8 => Bson::Int32({
                let l = get_meta_data()?.octet_length;
                match l {
                    None => definitions::SQL_NO_TOTAL as i32,
                    Some(l) => l as i32,
                }
            }),
            9 => Bson::Int32(get_meta_data()?.scale.unwrap_or(0) as i32),
            10 => match get_meta_data()?.sql_type {
                SqlDataType::SQL_INTEGER | SqlDataType::SQL_DOUBLE | SqlDataType::SQL_DECIMAL => {
                    Bson::Int32(10)
                }
                _ => Bson::Null,
            },
            11 => Bson::Int32(get_meta_data()?.nullability as i32),
            12 => Bson::String("".to_string()),
            13 => Bson::Null,
            14 => Bson::Int32(get_meta_data()?.non_concise_type as i32),
            15 => match get_meta_data()?.sql_code {
                None => Bson::Null,
                Some(x) => Bson::Int32(x as i32),
            },
            16 => Bson::Int32({
                let l = get_meta_data()?.octet_length;
                match l {
                    None => definitions::SQL_NO_TOTAL as i32,
                    Some(_) => 0i32,
                }
            }),
            17 => Bson::Int32(1 + self.current_field_for_collection as i32),
            18 => Bson::String(
                // odbc_sys should use an enum instead of constants...
                match get_meta_data()?.nullability {
                    Nullability::SQL_NULLABLE_UNKNOWN | Nullability::SQL_NULLABLE => "YES",
                    Nullability::SQL_NO_NULLS => "NO",
                }
                .to_string(),
            ),
            _ => return Err(Error::ColIndexOutOfBounds(col_index)),
        }))
    }

    fn get_resultset_metadata(&self) -> &Vec<crate::MongoColMetadata> {
        &FIELDS_METADATA
    }
}
