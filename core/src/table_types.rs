use crate::{databases::DATABASES_METADATA, err::Result, Error, MongoColMetadata, MongoStatement};
use bson::Bson;

#[derive(Debug)]
pub struct MongoTableTypes {
    // The list of all the table types
    table_type: Vec<String>,
    // The current table type index
    current_table_type_index: usize,
}

impl MongoTableTypes {
    // Statement for SQLTables("", "", "", SQL_ALL_TABLE_TYPES ).
    pub fn all_table_types() -> MongoTableTypes {
        MongoTableTypes {
            table_type: vec!["TABLE".to_string(), "VIEW".to_string()],
            current_table_type_index: 0,
        }
    }
}

impl MongoStatement for MongoTableTypes {
    // Increment current_table_type_index.
    // Return true if current_table_type_index index is <= for table_type.length.
    fn next(&mut self) -> Result<bool> {
        self.current_table_type_index += 1;
        Ok(self.current_table_type_index <= self.table_type.len())
    }

    // Get the BSON value for the value at the given colIndex on the current row.
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        // The mapping for col_index <-> Value will be hard-coded and handled in this function
        // 1..3 | 5-> Null
        // 4 -> table_type[current_table_type_index-1]
        match col_index {
            1..=3 | 5 => Ok(Some(Bson::Null)),
            4 => Ok(Some(Bson::String(
                self.table_type
                    .get(self.current_table_type_index - 1)
                    .unwrap()
                    .to_string(),
            ))),
            _ => Err(Error::ColIndexOutOfBounds(col_index)),
        }
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &*DATABASES_METADATA
    }
}
