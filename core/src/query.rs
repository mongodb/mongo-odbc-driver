use crate::cluster_type::MongoClusterType;
use crate::col_metadata::VersionedJsonSchema;
use crate::json_schema::Schema;
use crate::util::libmongosqltranslate_run_command;
use crate::{
    col_metadata::{MongoColMetadata, SqlGetSchemaResponse},
    conn::MongoConnection,
    err::Result,
    stmt::MongoStatement,
    Error, TypeMode,
};
use mongodb::{
    bson::{doc, document::ValueAccessError, Bson, Document},
    error::{CommandError, ErrorKind},
    Cursor, Database,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::Duration;

const BATCH_SIZE_REPLACEMENT_THRESHOLD: u32 = 100;

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

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Namespace {
    pub database: String,
    pub collection: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translation {
    pub target_db: String,
    pub target_collection: Option<String>,
    pub pipeline: bson::Bson,
    pub result_set_schema: Schema,
    pub select_order: Vec<Vec<String>>,
}

impl Translation {
    pub fn from_document(doc: &Document) -> Result<Self> {
        let as_bson = Bson::Document(doc.clone());
        let deserializer = bson::Deserializer::new(as_bson);
        let deserializer = serde_stacker::Deserializer::new(deserializer);
        Deserialize::deserialize(deserializer).map_err(Error::LibmongosqltranslateDeserialization)
    }
}

impl Namespace {
    pub fn from_bson(bson: Bson) -> Result<BTreeSet<Self>> {
        let deserializer = bson::Deserializer::new(bson);
        let deserializer = serde_stacker::Deserializer::new(deserializer);
        Deserialize::deserialize(deserializer).map_err(Error::LibmongosqltranslateDeserialization)
    }
}

impl MongoQuery {
    fn get_sql_query_namespaces(sql_query: &str, db: &String) -> Result<BTreeSet<Namespace>> {
        let get_namespaces_command = doc! {
            "command": "getNamespaces",
            "options": {
                "sql": sql_query,
                "db": db,
            },
        };

        let returned_doc = libmongosqltranslate_run_command(get_namespaces_command)?;

        let namespaces: BTreeSet<Namespace> = Namespace::from_bson(
            returned_doc
                .get("namespaces")
                .ok_or(Error::LibmongosqltranslateDocumentHasMissingKey(
                    "getNamespaces".to_string(),
                    "namespaces".to_string(),
                ))?
                .to_owned(),
        )?;

        Ok(namespaces)
    }

    fn translate_sql(
        sql_query: &str,
        current_db: &String,
        namespaces: BTreeSet<Namespace>,
        client: &MongoConnection,
        db: &Database,
    ) -> Result<Translation> {
        let schema_collection = db.collection::<Document>("__sql_schemas");

        // create the schema_catalog document
        let mut db_doc = doc! {};

        for namespace in namespaces {
            let namespace_schema_doc = client
                .runtime
                .block_on(async {
                    schema_collection
                        .find_one(doc! {
                            "_id": &namespace.collection
                        })
                        .await
                        .map_err(Error::QueryExecutionFailed)
                })?
                .ok_or(Error::SchemaDocumentNotFoundInSchemaCollection(
                    namespace.collection.clone(),
                ))?;

            let bson_schema = namespace_schema_doc.get("schema").ok_or(
                Error::SchemaCollectionDocumentHasMissingKey(
                    "schema".to_string(),
                    namespace.collection.clone(),
                ),
            )?;

            db_doc.insert(namespace.collection, bson_schema);
        }

        let schema_catalog_doc: Document = doc! {
            current_db: db_doc
        };

        let translate_command = doc! {
            "command": "translate",
            "options": {
                "sql": sql_query,
                "db": current_db,
                "excludeNamespaces": false,
                "relaxSchemaChecking": true,
                "schemaCatalog": schema_catalog_doc
            },
        };

        let returned_doc = libmongosqltranslate_run_command(translate_command)?;

        let mongosql_translation = Translation::from_document(&returned_doc)?;

        Ok(mongosql_translation)
    }

    // Create a MongoQuery with only the resultset_metadata.
    pub fn prepare(
        client: &MongoConnection,
        current_db: Option<String>,
        query_timeout: Option<u32>,
        query: &str,
        type_mode: TypeMode,
        max_string_length: Option<u16>,
    ) -> Result<Self> {
        let working_db = current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = client.client.database(working_db);

        let metadata = match client.cluster_type {
            MongoClusterType::AtlasDataFederation => {
                // 1. Run the sqlGetResultSchema command to get the result set
                // metadata. Column metadata is sorted alphabetically by table
                // and column name.
                let get_result_schema_cmd =
                    doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};

                let guard = client.runtime.enter();
                let schema_response = client.runtime.block_on(async {
                    db.run_command(get_result_schema_cmd)
                        .await
                        .map_err(Error::QueryExecutionFailed)
                })?;
                drop(guard);
                let get_result_schema_response: SqlGetSchemaResponse =
                    mongodb::bson::from_document(schema_response)
                        .map_err(Error::QueryDeserialization)?;

                get_result_schema_response.process_result_metadata(
                    working_db,
                    type_mode,
                    max_string_length,
                )?
            }
            MongoClusterType::Enterprise => {
                // Get relevant namespaces
                let namespaces: BTreeSet<Namespace> =
                    Self::get_sql_query_namespaces(query, working_db)?;

                // Translate sql
                let mongosql_translation =
                    Self::translate_sql(query, working_db, namespaces, client, &db)?;

                let translation_metadata = SqlGetSchemaResponse {
                    ok: 1,
                    schema: VersionedJsonSchema {
                        version: 1,
                        json_schema: mongosql_translation.result_set_schema,
                    },
                    select_order: Some(mongosql_translation.select_order),
                };

                translation_metadata.process_result_metadata(
                    working_db,
                    type_mode,
                    max_string_length,
                )?
            }
            MongoClusterType::Community | MongoClusterType::UnknownTarget => {
                // On connection, these types should get caught and throw an error.
                unreachable!()
            }
        };

        Ok(Self {
            resultset_cursor: None,
            resultset_metadata: metadata,
            current: None,
            current_db,
            query: query.to_string(),
            query_timeout,
        })
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self, connection: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        let guard = connection.unwrap().runtime.enter();
        let res = self
            .resultset_cursor
            .as_mut()
            .map_or(Err(Error::StatementNotExecuted), |c| {
                connection
                    .unwrap()
                    .runtime
                    .block_on(async { c.advance().await.map_err(Error::QueryCursorUpdate) })
            })?;
        drop(guard);
        // Cursor::advance must return Ok(true) before Cursor::deserialize_current can be invoked.
        // Calling Cursor::deserialize_current after Cursor::advance does not return true or without
        // calling Cursor::advance at all may result in a panic
        if res {
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

        Ok((res, vec![]))
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16, max_string_length: Option<u16>) -> Result<Option<Bson>> {
        let current = self.current.as_ref().ok_or(Error::InvalidCursorState)?;
        let md = self
            .get_col_metadata(col_index, max_string_length)
            .map_err(|_| Error::ColIndexOutOfBounds(col_index))?;
        let datasource = current
            .get_document(&md.table_name)
            .map_err(|e: ValueAccessError| Error::ValueAccess(col_index.to_string(), e))?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self, _: Option<u16>) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }

    // Execute the $sql aggregation for the query and initialize the result set
    // cursor. If there is a timeout, the query must finish before the timeout
    // or an error is returned.
    fn execute(
        &mut self,
        connection: &MongoConnection,
        stmt_id: Bson,
        rowset_size: u32,
    ) -> Result<bool> {
        let current_db = self.current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = connection.client.database(current_db);

        let (pipeline, collection) = match connection.cluster_type {
            MongoClusterType::AtlasDataFederation => {
                // 2. Run the $sql aggregation to get the result set cursor.
                let pipeline = vec![doc! {"$sql": {
                    "statement": &self.query,
                }}];
                (pipeline, None)
            }
            MongoClusterType::Enterprise => {
                // Get relevant namespaces
                let namespaces: BTreeSet<Namespace> =
                    Self::get_sql_query_namespaces(&self.query, current_db)?;

                // Translate sql
                let mongosql_translation =
                    Self::translate_sql(&self.query, current_db, namespaces, connection, &db)?;

                let pipeline = mongosql_translation
                    .pipeline
                    .as_array()
                    .ok_or(Error::TranslationPipelineNotArray)?
                    .iter()
                    .map(|bson_doc| {
                        bson_doc
                            .as_document()
                            .ok_or(Error::TranslationPipelineArrayContainsNonDocument)?
                            .to_owned()
                    })
                    .collect::<Vec<Document>>();

                (pipeline, mongosql_translation.target_collection)
            }
            MongoClusterType::Community | MongoClusterType::UnknownTarget => {
                // On connection, these types should get caught and throw an error.
                unreachable!()
            }
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

        let cursor: Cursor<Document> = if let Some(c) = collection {
            let collection = db.collection::<Document>(c.as_str());
            let mut aggregate = collection.aggregate(pipeline).comment(stmt_id);

            // If the query timeout is 0, it means "no timeout"
            if self.query_timeout.is_some_and(|timeout| timeout > 0) {
                aggregate = aggregate.max_time(Duration::from_millis(u64::from(
                    self.query_timeout.unwrap(),
                )));
            }

            // If rowset_size is large, then update the batch_size to be rowset_size for better efficiency.
            if rowset_size > BATCH_SIZE_REPLACEMENT_THRESHOLD {
                aggregate = aggregate.batch_size(rowset_size);
            }

            let _guard = connection.runtime.enter();
            connection
                .runtime
                .block_on(async { aggregate.await.map_err(map_query_error) })?
        } else {
            let mut aggregate = db.aggregate(pipeline).comment(stmt_id);

            // If the query timeout is 0, it means "no timeout"
            if self.query_timeout.is_some_and(|timeout| timeout > 0) {
                aggregate = aggregate.max_time(Duration::from_millis(u64::from(
                    self.query_timeout.unwrap(),
                )));
            }

            // If rowset_size is large, then update the batch_size to be rowset_size for better efficiency.
            if rowset_size > BATCH_SIZE_REPLACEMENT_THRESHOLD {
                aggregate = aggregate.batch_size(rowset_size);
            }

            let _guard = connection.runtime.enter();
            connection
                .runtime
                .block_on(async { aggregate.await.map_err(map_query_error) })?
        };

        self.resultset_cursor = Some(cursor);
        Ok(true)
    }

    // Close the cursor by setting the current value and cursor to None.
    fn close_cursor(&mut self) {
        self.current = None;
        self.resultset_cursor = None;
    }
}
