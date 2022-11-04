use crate::{
    col_metadata::{ColumnNullability, MongoColMetadata},
    conn::MongoConnection,
    err::Result,
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    stmt::MongoStatement,
    Error,
};
use bson::Bson;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref DATABASES_METADATA: Vec<MongoColMetadata> = vec![
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
            ColumnNullability::Nullable
        ),
        MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_TYPE".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            ColumnNullability::Nullable
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
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        assert_eq!(5, MongoDatabases::empty().get_resultset_metadata().len());
    }

    #[test]
    fn metadata_column_names() {
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        // This gives us assurance that the column names are all correct.
        assert_eq!(
            "TABLE_CAT",
            MongoDatabases::empty()
                .get_col_metadata(1)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_SCHEM",
            MongoDatabases::empty()
                .get_col_metadata(2)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_NAME",
            MongoDatabases::empty()
                .get_col_metadata(3)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "TABLE_TYPE",
            MongoDatabases::empty()
                .get_col_metadata(4)
                .unwrap()
                .col_name
        );
        assert_eq!(
            "REMARKS",
            MongoDatabases::empty()
                .get_col_metadata(5)
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
                .get_col_metadata(1)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(2)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(3)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(4)
                .unwrap()
                .type_name
        );
        assert_eq!(
            "string",
            MongoDatabases::empty()
                .get_col_metadata(5)
                .unwrap()
                .type_name
        );
    }

    #[test]
    fn metadata_column_nullability() {
        use crate::col_metadata::ColumnNullability;
        use crate::{databases::MongoDatabases, stmt::MongoStatement};
        assert_eq!(
            ColumnNullability::NoNulls,
            MongoDatabases::empty()
                .get_col_metadata(1)
                .unwrap()
                .is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            MongoDatabases::empty()
                .get_col_metadata(2)
                .unwrap()
                .is_nullable
        );
        // Docs do not say NoNulls, but there is no way the tale name can be null.
        assert_eq!(
            ColumnNullability::Nullable,
            MongoDatabases::empty()
                .get_col_metadata(3)
                .unwrap()
                .is_nullable
        );
        // The docs also do not say NoNulls, but they enumerate every possible value and
        // NULL is not one of them.
        assert_eq!(
            ColumnNullability::Nullable,
            MongoDatabases::empty()
                .get_col_metadata(4)
                .unwrap()
                .is_nullable
        );
        assert_eq!(
            ColumnNullability::Nullable,
            MongoDatabases::empty()
                .get_col_metadata(5)
                .unwrap()
                .is_nullable
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
    fn next(&mut self) -> Result<bool> {
        self.current_db_index += 1;
        Ok(self.current_db_index <= self.database_names.len())
    }

    // Get the BSON value for the value at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1-> databases_names[current_row_index]
        // 2..5 -> Null
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

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &DATABASES_METADATA
    }
}
