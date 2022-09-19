use crate::err::Result;
use bson::Bson;
use std::fmt::Debug;

pub trait MongoStatement {
    // Move the cursor to the next item.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool>;
    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row has not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>>;
}

impl Debug for dyn MongoStatement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MongoStatement")
    }
}
