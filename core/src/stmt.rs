use crate::{
    err::{Error, Result},
    MongoColMetadata, MongoConnection,
};
use bson::Bson;
use std::fmt::Debug;

pub trait MongoStatement: Debug {
    // Move the cursor to the next item.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, mongo_connection: Option<&MongoConnection>) -> Result<bool>;
    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>>;
    // Return a reference to the ResultSetMetadata for this Statement.
    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata>;
    // get_col_metadata gets the metadata for a given column, 1-indexed as per the ODBC spec.
    fn get_col_metadata(&self, col_index: u16) -> Result<&MongoColMetadata> {
        if col_index == 0 {
            return Err(Error::ColIndexOutOfBounds(0));
        }
        self.get_resultset_metadata()
            .get((col_index - 1) as usize)
            .map_or(Err(Error::ColIndexOutOfBounds(col_index)), Ok)
    }
}

#[derive(Debug)]
pub struct EmptyStatement {
    pub resultset_metadata: Vec<MongoColMetadata>,
}

impl MongoStatement for EmptyStatement {
    fn next(&mut self, _mongo_connection: Option<&MongoConnection>) -> Result<bool> {
        Ok(false)
    }

    fn get_value(&self, _col_index: u16) -> Result<Option<Bson>> {
        Err(Error::InvalidCursorState)
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }
}

#[cfg(test)]
mod unit {
    use crate::{
        col_metadata::MongoColMetadata,
        json_schema::{
            simplified::{Atomic, Schema},
            BsonTypeName,
        },
        stmt::{EmptyStatement, MongoStatement},
    };
    use odbc_sys::Nullability;

    #[test]
    fn metadata_column_names() {
        let test_empty = EmptyStatement {
            resultset_metadata: vec![MongoColMetadata::new(
                "",
                "".to_string(),
                "TABLE_CAT".to_string(),
                Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
                Nullability::NO_NULLS,
            )],
        };

        assert_eq!(
            "TABLE_CAT",
            test_empty.get_col_metadata(1).unwrap().col_name
        );
    }
}
