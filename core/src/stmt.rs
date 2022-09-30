use crate::{err::Result, MongoColMetadata};
use bson::Bson;
use std::fmt::Debug;

pub trait MongoStatement: Debug {
    // Move the cursor to the next item.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool>;
    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>>;
    // Return a reference to the ResultSetMetadata for this Statement.
    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata>;
}
