use crate::{
    col_metadata::MongoColMetadata, conn::MongoConnection, err::Result, stmt::MongoStatement,
    util::DISALLOWED_DB_NAMES, BsonTypeInfo, Error,
};
use definitions::Nullability;
use mongodb::bson::Bson;

use once_cell::sync::OnceCell;

pub(crate) static DATABASES_METADATA: OnceCell<Vec<MongoColMetadata>> = OnceCell::new();

pub(crate) fn init_databases_metadata(max_string_length: Option<u16>) -> Vec<MongoColMetadata> {
    vec![
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            BsonTypeInfo::STRING,
            max_string_length,
            Nullability::SQL_NO_NULLS,
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_SCHEM".to_string(),
            BsonTypeInfo::STRING,
            max_string_length,
            Nullability::SQL_NULLABLE,
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_NAME".to_string(),
            BsonTypeInfo::STRING,
            max_string_length,
            Nullability::SQL_NULLABLE,
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "TABLE_TYPE".to_string(),
            BsonTypeInfo::STRING,
            max_string_length,
            Nullability::SQL_NULLABLE,
        ),
        MongoColMetadata::new_metadata_from_bson_type_info_default(
            "",
            "".to_string(),
            "REMARKS".to_string(),
            BsonTypeInfo::STRING,
            max_string_length,
            Nullability::SQL_NULLABLE,
        ),
    ]
}

mod unit {
    #[test]
    fn metadata_size() {
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        assert_eq!(
            5,
            MongoDatabases::empty().get_resultset_metadata(None).len()
        );
    }

    #[test]
    fn metadata_column_names() {
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        // This gives us assurance that the column names are all correct.
        assert_eq!(
            "TABLE_CAT",
            MongoDatabases::empty()
                .get_col_metadata(1, None)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_SCHEM",
            MongoDatabases::empty()
                .get_col_metadata(2, None)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_NAME",
            MongoDatabases::empty()
                .get_col_metadata(3, None)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_TYPE",
            MongoDatabases::empty()
                .get_col_metadata(4, None)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "REMARKS",
            MongoDatabases::empty()
                .get_col_metadata(5, None)
                .unwrap()
                .col_name
        );
    }

    #[test]
    fn metadata_column_types() {
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(1, None)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(2, None)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(3, None)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(4, None)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(5, None)
                .unwrap()
                .type_name
        );
    }

    #[test]
    fn metadata_column_nullability() {
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        use definitions::Nullability;
        assert_eq!(
            Nullability::SQL_NO_NULLS,
            MongoDatabases::empty()
                .get_col_metadata(1, None)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::SQL_NULLABLE,
            MongoDatabases::empty()
                .get_col_metadata(2, None)
                .unwrap()
                .nullability
        );
        // Docs do not say NO_NULLS, but there is no way the tale name can be null.
        assert_eq!(
            Nullability::SQL_NULLABLE,
            MongoDatabases::empty()
                .get_col_metadata(3, None)
                .unwrap()
                .nullability
        );
        // The docs also do not say NO_NULLS, but they enumerate every possible value and
        // NULL is not one of them.
        assert_eq!(
            Nullability::SQL_NULLABLE,
            MongoDatabases::empty()
                .get_col_metadata(4, None)
                .unwrap()
                .nullability
        );
        assert_eq!(
            Nullability::SQL_NULLABLE,
            MongoDatabases::empty()
                .get_col_metadata(5, None)
                .unwrap()
                .nullability
        );
    }
}

#[derive(Debug)]
pub struct MongoDatabases {
    // The list of all the databases
    database_names: Vec<String>,
    // The current database index.
    current_db_index: usize,
}

// Statement for SQLTables(SQL_ALL_CATALOGS, "","").
impl MongoDatabases {
    // Create a new MongoStatement to list all the valid databases.
    // Correspond to SQLTables(SQL_ALL_CATALOGS, "","").
    // All columns except the TABLE_CAT column contain NULLs.
    // The query timeout comes from the statement attribute SQL_ATTR_QUERY_TIMEOUT. If there is a
    // timeout, the query must finish before the timeout or an error is returned.
    pub fn list_all_catalogs(
        mongo_connection: &MongoConnection,
        _query_timeout: Option<i32>,
    ) -> Self {
        let _guard = mongo_connection.runtime.enter();
        let database_names: Vec<String> = mongo_connection
            .runtime
            .block_on(async {
                mongo_connection
                    .client
                    .list_database_names()
                    .authorized_databases(true)
                    .await
            })
            .unwrap()
            .iter()
            .filter(|&db_name| !DISALLOWED_DB_NAMES.contains(&db_name.as_str()))
            .map(|s| s.to_string())
            .collect();

        MongoDatabases {
            database_names,
            current_db_index: 0,
        }
    }

    pub fn empty() -> MongoDatabases {
        MongoDatabases {
            database_names: vec![],
            current_db_index: 0,
        }
    }
}

impl MongoStatement for MongoDatabases {
    // Increment current_db_index.
    // Return true if current_db_index index is <= for databases_names.length.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        self.current_db_index += 1;
        Ok((self.current_db_index <= self.database_names.len(), vec![]))
    }

    // Get the BSON value for the value at the given colIndex on the current row.
    fn get_value(&self, col_index: u16, _: Option<u16>) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> databases_names[current_row_index]
        // 2..=4 -> Null
        // 5 => "" (Remarks)
        match col_index {
            1 => Ok(Some(Bson::String(
                self.database_names
                    .get(self.current_db_index - 1)
                    .unwrap()
                    .to_string(),
            ))),
            2..=5 => Ok(Some(Bson::Null)),
            _ => Err(Error::ColIndexOutOfBounds(col_index)),
        }
    }

    fn get_resultset_metadata(&self, max_string_length: Option<u16>) -> &Vec<MongoColMetadata> {
        DATABASES_METADATA.get_or_init(|| init_databases_metadata(max_string_length))
    }
}
