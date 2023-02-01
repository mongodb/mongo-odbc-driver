use crate::{
    col_metadata::{MongoColMetadata, SqlGetSchemaResponse},
    conn::MongoConnection,
    err::Result,
    stmt::MongoStatement,
    Error,
};
use bson::{doc, Bson, Document};
use mongodb::{options::AggregateOptions, sync::Cursor};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Cursor<Document>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current deserialized "row".
    current: Option<Document>,
}

impl MongoQuery {
    // Create a new MongoQuery on the connection's current database. Execute a
    // $sql aggregation with the given query and initialize the result set
    // cursor. If there is a timeout, the query must finish before the timeout
    // or an error is returned.
    pub fn execute(
        client: &MongoConnection,
        query_timeout: Option<u32>,
        query: &str,
    ) -> Result<Self> {
        let current_db = client.current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = client.client.database(current_db);

        // 1. Run the sqlGetResultSchema command to get the result set
        // metadata. Column metadata is sorted alphabetically by table
        // and column name.
        let get_result_schema_cmd =
            doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};

        let get_result_schema_response: SqlGetSchemaResponse =
            bson::from_document(db.run_command(get_result_schema_cmd, None)?)
                .map_err(Error::BsonDeserialization)?;

        let metadata = get_result_schema_response.process_result_metadata(current_db)?;

        // 2. Run the $sql aggregation to get the result set cursor.
        let pipeline = vec![doc! {"$sql": {
            "format": "odbc",
            "formatVersion": 1,
            "statement": query,
        }}];

        let cursor: Cursor<Document> = match query_timeout {
            Some(i) => {
                if i > 0 {
                    let opt = AggregateOptions::builder()
                        .max_time(Duration::from_millis(i as u64))
                        .build();
                    db.aggregate(pipeline, opt)?
                } else {
                    // If the query timeout is 0, it means "no timeout"
                    db.aggregate(pipeline, None)?
                }
            }
            _ => db.aggregate(pipeline, None)?,
        };
        Ok(MongoQuery {
            resultset_cursor: cursor,
            resultset_metadata: metadata,
            current: None,
        })
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<bool> {
        let res = self.resultset_cursor.advance().map_err(Error::Mongo);
        if let Ok(false) = res {
            // deserialize_current unwraps None if we do not check the value of advance.
            self.current = None;
            return res;
        }
        self.current = Some(self.resultset_cursor.deserialize_current()?);
        res
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let current = self.current.as_ref().ok_or(Error::InvalidCursorState)?;
        let md = self.get_col_metadata(col_index)?;
        let datasource = current
            .get_document(&md.table_name)
            .map_err(Error::ValueAccess)?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }
}
