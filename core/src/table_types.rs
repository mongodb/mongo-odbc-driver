use crate::{
    databases::{SIMPLE_DATABASES_METADATA, STANDARD_DATABASES_METADATA},
    err::Result,
    Error, MongoColMetadata, MongoConnection, MongoStatement, TypeMode,
};
use bson::Bson;

const TABLE_TYPES: [&str; 2] = ["TABLE", "VIEW"];

#[derive(Debug)]
pub struct MongoTableTypes {
    // The list of all the table types
    table_type: Vec<&'static str>,
    // The current table type index
    current_table_type_index: usize,
    type_mode: TypeMode,
}

impl MongoTableTypes {
    // Statement for SQLTables("", "", "", SQL_ALL_TABLE_TYPES ).
    pub fn all_table_types(type_mode: TypeMode) -> MongoTableTypes {
        MongoTableTypes {
            table_type: Vec::from(TABLE_TYPES),
            current_table_type_index: 0,
            type_mode,
        }
    }
}

impl MongoStatement for MongoTableTypes {
    // Increment current_table_type_index.
    // Return true if current_table_type_index index is <= for table_type.length.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        self.current_table_type_index += 1;
        Ok((
            self.current_table_type_index <= self.table_type.len(),
            vec![],
        ))
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
        match self.type_mode {
            TypeMode::Standard => &STANDARD_DATABASES_METADATA,
            TypeMode::Simple => &SIMPLE_DATABASES_METADATA,
        }
    }
}
