use crate::{
    cluster_type::MongoClusterType,
    col_metadata::{MongoColMetadata, ResultSetSchema, SqlGetSchemaResponse},
    conn::MongoConnection,
    err::Result,
    mongosqltranslate::{
        libmongosqltranslate_run_command, CommandResponse, GetNamespaces, Namespace, Translate,
        TranslateCommandResponse,
    },
    stmt::MongoStatement,
    Error, TypeMode,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, document::ValueAccessError, Bson, Document},
    error::{CommandError, ErrorKind},
    Cursor, Database,
};
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
    // The current collection. Only used in Enterprise mode.
    pub current_collection: Option<String>,
    // The MQL aggregation pipeline
    pub pipeline: Vec<Document>,
    // The query timeout
    pub query_timeout: Option<u32>,
}

impl MongoQuery {
    fn get_sql_query_namespaces(sql_query: &str, db: &String) -> Result<BTreeSet<Namespace>> {
        let command = GetNamespaces::new(sql_query.to_string(), db.to_string());

        let command_response = libmongosqltranslate_run_command(command)?;

        if let CommandResponse::GetNamespaces(response) = command_response {
            Ok(response.namespaces)
        } else {
            unreachable!()
        }
    }

    fn translate_sql(
        sql_query: &str,
        current_db: &String,
        namespaces: BTreeSet<Namespace>,
        client: &MongoConnection,
        db: &Database,
    ) -> Result<TranslateCommandResponse> {
        let schema_collection = db.collection::<Document>("__sql_schemas");

        let collection_names = namespaces
            .iter()
            .map(|namespace| namespace.collection.as_str())
            .collect::<Vec<&str>>();

        // Create an aggregation pipeline to fetch the schema information for the specified collections.
        // The pipeline uses $in to query all the specified collections and projects them into the desired format:
        // "dbName": { "collection1" : "Schema1", "collection2" : "Schema2", ... }
        let schema_catalog_aggregation_pipeline = vec![
            doc! {"$match": {
                "_id": {
                    "$in": collection_names
                    }
                }
            },
            doc! {"$project":{
                "_id": 1,
                "schema": 1
                }
            },
            doc! {"$group": {
                "_id": null,
                "collections": {
                    "collectionName": "$_id",
                    "schema": "$schema"
                    }
                }
            },
            doc! {"$project": {
                "_id": 0,
                current_db: {
                    "$arrayToObject": [{
                        "$map": {
                            "input": "$collections",
                            "as": "coll",
                            "in": {
                                "k": "$$coll.collectionName",
                                "v": "$$coll.schema"
                                }
                            }
                        }]
                    }
                }
            },
        ];

        // create the schema_catalog document
        let schema_catalog_doc_vec: Vec<Document> = client.runtime.block_on(async {
            schema_collection
                .aggregate(schema_catalog_aggregation_pipeline)
                .await
                .map_err(Error::QueryExecutionFailed)?
                .try_collect::<Vec<Document>>()
                .await
                .map_err(Error::QueryExecutionFailed)
        })?;

        if schema_catalog_doc_vec.len() > 1 {
            return Err(Error::MultipleSchemaDocumentsReturned(
                schema_catalog_doc_vec.len(),
            ));
        } else if schema_catalog_doc_vec.is_empty() {
            return Err(Error::NoSchemaInformationReturned);
        }

        let schema_catalog_doc = schema_catalog_doc_vec[0].to_owned();

        let collections_schema_doc = schema_catalog_doc
            .get_document(current_db)
            .map_err(|e: ValueAccessError| Error::ValueAccess(current_db.to_string(), e))?;

        if namespaces.len() != collections_schema_doc.len() {
            let missing_collections: Vec<String> = namespaces
                .iter()
                .map(|namespace| namespace.collection.clone())
                .filter(|collection| !collections_schema_doc.contains_key(collection.as_str()))
                .collect();

            return Err(Error::SchemaDocumentNotFoundInSchemaCollection(
                missing_collections,
            ));
        }

        let command = Translate::new(
            sql_query.to_string(),
            current_db.to_string(),
            schema_catalog_doc,
        );

        let command_response = libmongosqltranslate_run_command(command)?;

        if let CommandResponse::Translate(response) = command_response {
            Ok(response)
        } else {
            unreachable!()
        }
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

        let (pipeline, current_collection, result_set_schema) = match client.cluster_type {
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

                // 2. Generate the $sql aggregation pipeline to use at execution time.
                let pipeline = vec![doc! {"$sql": {
                    "statement": query,
                }}];

                (
                    pipeline,
                    None,
                    ResultSetSchema::from(get_result_schema_response),
                )
            }
            MongoClusterType::Enterprise => {
                // Get relevant namespaces
                let namespaces: BTreeSet<Namespace> =
                    Self::get_sql_query_namespaces(query, working_db)?;

                // Translate sql
                let mongosql_translation =
                    Self::translate_sql(query, working_db, namespaces, client, &db)?;

                let mut pipeline: Vec<Document> = Vec::new();

                for bson_doc in mongosql_translation
                    .pipeline
                    .as_array()
                    .ok_or(Error::TranslationPipelineNotArray)?
                    .iter()
                {
                    match bson_doc.as_document() {
                        None => return Err(Error::TranslationPipelineArrayContainsNonDocument),
                        Some(doc) => pipeline.push(doc.to_owned()),
                    }
                }

                (
                    pipeline,
                    mongosql_translation.target_collection,
                    mongosql_translation.result_set_schema,
                )
            }
            MongoClusterType::Community | MongoClusterType::UnknownTarget => {
                // On connection, these types should get caught and throw an error.
                unreachable!()
            }
        };

        let metadata =
            result_set_schema.process_result_metadata(working_db, type_mode, max_string_length)?;

        Ok(Self {
            resultset_cursor: None,
            resultset_metadata: metadata,
            current: None,
            current_db,
            current_collection,
            pipeline,
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

        let collection;
        let mut aggregate = if let Some(c_name) = self.current_collection.as_ref() {
            collection = db.collection::<Document>(c_name);
            collection.aggregate(self.pipeline.to_owned())
        } else {
            db.aggregate(self.pipeline.to_owned())
        };

        aggregate = aggregate.comment(stmt_id);

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

        // handle an error coming back from execution; if it was cancelled, throw a specific error to
        // denote this to the program, otherwise return a generic query execution error
        let map_query_error = |e: mongodb::error::Error| match *e.kind {
            ErrorKind::Command(CommandError {
                code: 11601, // interrupted
                ..
            }) => Error::QueryCancelled,
            _ => Error::QueryExecutionFailed(e),
        };

        let _guard = connection.runtime.enter();
        let cursor: Cursor<Document> = connection
            .runtime
            .block_on(async { aggregate.await.map_err(map_query_error) })?;

        self.resultset_cursor = Some(cursor);
        Ok(true)
    }

    // Close the cursor by setting the current value and cursor to None.
    fn close_cursor(&mut self) {
        self.current = None;
        self.resultset_cursor = None;
    }
}
