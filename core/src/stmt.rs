use crate::error::Error;
use bson::Bson;

pub trait MongoStatement {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool, Error>;
    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<&Bson>, Error>;
}
