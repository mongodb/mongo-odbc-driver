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

lazy_static! {
    // TODO: It isn't currently clear what the difference between collections and databases is since
    // the ODBC standard does not have a SQLDatabases (or SQLCatalogs) call, just SQLTables
    // and SQLColumns.
    // Do we actually need both of these?
    static ref DATABASES_METADATA: Vec<MongoColMetadata> = vec![
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
        assert_eq!(5, super::DATABASES_METADATA.len());
    }

    #[test]
    fn metadata_column_names() {
        // These were generated straight from the docs (hence the - 1). This
        // gives us assurance that the column names are all correct.
        assert_eq!("TABLE_CAT", super::DATABASES_METADATA[1 - 1].col_name);
        assert_eq!("TABLE_SCHEM", super::DATABASES_METADATA[2 - 1].col_name);
        assert_eq!("TABLE_NAME", super::DATABASES_METADATA[3 - 1].col_name);
        assert_eq!("TABLE_TYPE", super::DATABASES_METADATA[4 - 1].col_name);
        assert_eq!("REMARKS", super::DATABASES_METADATA[5 - 1].col_name);
    }

    #[test]
    fn metadata_column_types() {
        // These were generated straight from the docs (hence the - 1).
        assert_eq!("string", super::DATABASES_METADATA[1 - 1].type_name);
        assert_eq!("string", super::DATABASES_METADATA[2 - 1].type_name);
        assert_eq!("string", super::DATABASES_METADATA[3 - 1].type_name);
        assert_eq!("string", super::DATABASES_METADATA[4 - 1].type_name);
        assert_eq!("string", super::DATABASES_METADATA[5 - 1].type_name);
    }

    fn metadata_column_nullability() {
        use crate::col_metadata::ColumnNullability;
        // These were generated straight from the docs (hence the - 1).
        assert_eq!(
            ColumnNullability::Nullable,
            super::DATABASES_METADATA[1 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::DATABASES_METADATA[2 - 1].is_nullable
        );
        // Docs do not say NoNulls, but there is no way the tale name can be null.
        assert_eq!(
            ColumnNullability::NoNulls,
            super::DATABASES_METADATA[3 - 1].is_nullable
        );
        // The docs also do not say NoNulls, but they enumerate every possible value and
        // NULL is not one of them.
        assert_eq!(
            ColumnNullability::NoNulls,
            super::DATABASES_METADATA[4 - 1].is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            super::DATABASES_METADATA[5 - 1].is_nullable
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
        let database_names: Vec<String> = mongo_connection
            .client
            .list_database_names(None, None)
            .unwrap();
        MongoDatabases {
            database_names,
            current_db_index: 0,
        }
    }
}

impl MongoStatement for MongoDatabases {
    // Increment current_db_index.
    // Return true if current_db_index index is <= for databases_names.length.
    fn next(&mut self) -> Result<bool> {
        self.current_db_index += 1;
        Ok(self.current_db_index <= self.database_names.len())
    }

    // Get the BSON value for the value at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> databases_names[current_row_index]
        match col_index {
            1 => Ok(Some(Bson::String(
                self.database_names
                    .get(self.current_db_index - 1)
                    .unwrap()
                    .to_string(),
            ))),
            _ => Ok(Some(Bson::Null)),
            // SQL-1031: Add database listing edge case handling
            // Col_or_Param_Num was greater than the number of columns in the result set
            // Or value specified for the argument Col_or_Param_Num was 0,
            // and the SQL_ATTR_USE_BOOKMARKS statement attribute was set to SQL_UB_OFF
            // Throw error 07009
        }
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &*DATABASES_METADATA
    }
}
