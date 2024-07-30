use crate::{
    err::{Error, Result},
    MongoColMetadata, MongoConnection,
};
use mongodb::bson::Bson;
use std::fmt::Debug;

pub trait MongoStatement: Debug {
    // Move the cursor to the next item.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, mongo_connection: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)>;
    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16, max_string_length: Option<u16>) -> Result<Option<Bson>>;
    // Return a reference to the ResultSetMetadata for this Statement.
    fn get_resultset_metadata(&self, max_string_length: Option<u16>) -> &Vec<MongoColMetadata>;
    // get_col_metadata gets the metadata for a given column, 1-indexed as per the ODBC spec.
    fn get_col_metadata(
        &self,
        col_index: u16,
        max_string_length: Option<u16>,
    ) -> Result<&MongoColMetadata> {
        if col_index == 0 {
            return Err(Error::ColIndexOutOfBounds(0));
        }
        self.get_resultset_metadata(max_string_length)
            .get((col_index - 1) as usize)
            .ok_or(Error::ColIndexOutOfBounds(col_index))
    }
    // Executes a prepared statement.
    // Only MongoQuery supports this workflow. The other statements don't.
    fn execute(
        &mut self,
        _connection: &MongoConnection,
        _stmt_id: Bson,
        _rowset_size: u32,
    ) -> Result<bool> {
        Err(Error::UnsupportedOperation("execute"))
    }
    // Closes the cursor.
    // Only MongoQuery supports this workflow. The other statements don't.
    fn close_cursor(&mut self) {}
}

#[derive(Debug)]
pub struct EmptyStatement {
    pub resultset_metadata: &'static Vec<MongoColMetadata>,
}

impl MongoStatement for EmptyStatement {
    fn next(&mut self, _mongo_connection: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        Ok((false, vec![]))
    }

    fn get_value(&self, _: u16, _: Option<u16>) -> Result<Option<Bson>> {
        Err(Error::InvalidCursorState)
    }

    fn get_resultset_metadata(&self, _: Option<u16>) -> &Vec<MongoColMetadata> {
        self.resultset_metadata
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
        TypeMode,
    };
    use definitions::Nullability;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref EMPTY_TEST_METADATA: Vec<MongoColMetadata> = vec![MongoColMetadata::new(
            "",
            "".to_string(),
            "TABLE_CAT".to_string(),
            Schema::Atomic(Atomic::Scalar(BsonTypeName::String)),
            Nullability::SQL_NO_NULLS,
            TypeMode::Standard,
            None,
        )];
    }

    #[test]
    fn empty_statement_correctness() {
        let mut test_empty = EmptyStatement {
            resultset_metadata: &EMPTY_TEST_METADATA,
        };

        assert_eq!(
            "TABLE_CAT",
            test_empty.get_col_metadata(1, None).unwrap().col_name
        );
        assert!(!test_empty.next(None).unwrap().0);
        assert!(test_empty.get_value(1, None).is_err());
    }
}
