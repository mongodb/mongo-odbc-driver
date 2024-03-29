use crate::{
    col_metadata::{MongoColMetadata, SqlGetSchemaResponse},
    conn::MongoConnection,
    err::Result,
    stmt::MongoStatement,
    Error, TypeMode,
};
use bson::{doc, document::ValueAccessError, Bson, Document};
use log::warn;
use mongodb::{
    error::{CommandError, ErrorKind},
    options::AggregateOptions,
    Cursor,
};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Option<Cursor<Document>>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current deserialized "row".
    current: Option<Document>,
    // The current database
    pub current_db: Option<String>,
    // The query
    pub query: String,
    // The query timeout
    pub query_timeout: Option<u32>,
}

impl MongoQuery {
    // Create a MongoQuery with only the resultset_metadata.
    pub fn prepare(
        client: &MongoConnection,
        current_db: Option<String>,
        query_timeout: Option<u32>,
        query: &str,
        type_mode: TypeMode,
    ) -> Result<Self> {
        client.runtime.block_on(async {
            let current_db = current_db.ok_or(Error::NoDatabase)?;
            let db = client.client.database(&current_db);

            // 1. Run the sqlGetResultSchema command to get the result set
            // metadata. Column metadata is sorted alphabetically by table
            // and column name.
            let get_result_schema_cmd =
                doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};

            let get_result_schema_response: SqlGetSchemaResponse = bson::from_document(
                db.run_command(get_result_schema_cmd, None)
                    .await
                    .map_err(Error::QueryExecutionFailed)?,
            )
            .map_err(Error::QueryDeserialization)?;

            let metadata =
                get_result_schema_response.process_result_metadata(&current_db, type_mode)?;

            Ok(Self {
                resultset_cursor: None,
                resultset_metadata: metadata,
                current: None,
                current_db: Some(current_db),
                query: query.to_string(),
                query_timeout,
            })
        })
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self, connection: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        let res = if let Some(c) = connection {
            let _guard = c.runtime.enter();
            let handle = c.runtime.handle();
            self.resultset_cursor
                .as_mut()
                .map_or(Err(Error::StatementNotExecuted), |c| {
                    handle.block_on(async { c.advance().await.map_err(Error::QueryCursorUpdate) })
                })
        } else {
            Err(Error::QueryCancelled)
        };
        // Cursor::advance must return Ok(true) before Cursor::deserialize_current can be invoked.
        // Calling Cursor::deserialize_current after Cursor::advance does not return true or without
        // calling Cursor::advance at all may result in a panic
        if let Ok(true) = res {
            self.current = Some(
                self.resultset_cursor
                    .as_ref()
                    .unwrap()
                    .deserialize_current()
                    .map_err(Error::QueryCursorUpdate)?,
            );
        } else {
            self.current = None;
        }

        Ok((res?, vec![]))
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let current = self.current.as_ref().ok_or(Error::InvalidCursorState)?;
        let md = self
            .get_col_metadata(col_index)
            .map_err(|_| Error::ColIndexOutOfBounds(col_index))?;
        let datasource = current
            .get_document(&md.table_name)
            .map_err(|e: ValueAccessError| Error::ValueAccess(col_index.to_string(), e))?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }

    // Execute the $sql aggregation for the query and initialize the result set
    // cursor. If there is a timeout, the query must finish before the timeout
    // or an error is returned.
    fn execute(&mut self, connection: &MongoConnection, stmt_id: Bson) -> Result<bool> {
        let current_db = self.current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = connection.client.database(current_db);

        // 2. Run the $sql aggregation to get the result set cursor.
        let pipeline = vec![doc! {"$sql": {
            "statement": &self.query,
        }}];

        log::warn!("Executing query: {}", self.query);

        let opt = AggregateOptions::builder().comment_bson(Some(stmt_id));
        // If the query timeout is 0, it means "no timeout"
        let options = if self.query_timeout.is_some_and(|timeout| timeout > 0) {
            opt.max_time(Duration::from_millis(self.query_timeout.unwrap() as u64))
                .build()
        } else {
            opt.build()
        };

        // handle an error coming back from execution; if it was cancelled, throw a specific error to
        // denote this to the program, otherwise return a generic query execution error
        let map_query_error = |e: mongodb::error::Error| match *e.kind {
            ErrorKind::Command(CommandError {
                code: 11601, // interrupted
                ..
            }) => Error::QueryCancelled,
            _ => Error::QueryExecutionFailed(e),
        };

        log::warn!("taking a tokio guard before executing query");

        let _guard = connection.runtime.enter();
        let cursor: Cursor<Document> = connection.runtime.block_on(async {
            db.aggregate(pipeline, options)
                .await
                .map_err(map_query_error)
        })?;
        warn!("cursor created");
        self.resultset_cursor = Some(cursor);
        Ok(true)
    }

    // Close the cursor by setting the current value and cursor to None.
    fn close_cursor(&mut self) {
        self.current = None;
        self.resultset_cursor = None;
    }
}
