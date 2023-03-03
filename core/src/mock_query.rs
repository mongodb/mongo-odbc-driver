use crate::{
    col_metadata::MongoColMetadata, err::Result, stmt::MongoStatement, Error, MongoConnection,
};
use bson::{document::ValueAccessError, Bson, Document};

#[derive(Debug, Clone)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset: Vec<Document>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current index in the resultset.
    current: Option<usize>,
}

impl MongoQuery {
    pub fn new(resultset: Vec<Document>, resultset_metadata: Vec<MongoColMetadata>) -> Self {
        MongoQuery {
            resultset,
            resultset_metadata,
            current: None,
        }
    }
}

impl MongoStatement for MongoQuery {
    // Move the current index to the next Document in the Vec.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        if let Some(current) = self.current {
            self.current = Some(current + 1);
        } else {
            self.current = Some(0);
        }
        let current = self.current.unwrap();
        if current < self.resultset.len() {
            return Ok((true, vec![]));
        }
        Ok((false, vec![]))
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before get_value).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let md = self.get_col_metadata(col_index)?;
        let datasource = self.resultset[self.current.ok_or(Error::InvalidCursorState)?]
            .get_document(&md.table_name)
            .map_err(|e: ValueAccessError| Error::ValueAccess(col_index.to_string(), e))?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }
}
