use crate::odbc_uri::UserOptions;
use crate::{err::Result, Error};
use crate::{MongoQuery, TypeMode};
use bson::{doc, Bson, UuidRepresentation};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
#[repr(C)]
pub struct MongoConnection {
    /// The mongo DB client
    pub client: Client,
    /// Number of seconds to wait for any request on the connection to complete before returning to
    /// the application.
    /// Comes from SQL_ATTR_CONNECTION_TIMEOUT if set. Used any time there is a time out in a
    /// situation not associated with query execution or login.
    pub operation_timeout: Option<Duration>,
    /// The UuidRepresentation to use for this connection.
    pub uuid_repr: Option<UuidRepresentation>,

    /// the tokio runtime
    pub runtime: tokio::runtime::Runtime,
}

impl MongoConnection {
    /// Creates a new MongoConnection with the given settings and runs a command to make
    /// sure that the MongoConnection is valid.
    ///
    /// The operation will timeout if it takes more than loginTimeout seconds. This timeout is
    /// delegated to the mongo rust driver.
    ///
    /// The initial current database if provided should come from SQL_ATTR_CURRENT_CATALOG
    /// and will take precedence over the database setting specified in the uri if any.
    /// The initial operation time if provided should come from and will take precedence over the
    /// setting specified in the uri if any.
    pub fn connect(
        mut user_options: UserOptions,
        current_db: Option<String>,
        operation_timeout: Option<u32>,
        login_timeout: Option<u32>,
        type_mode: TypeMode,
        mut runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<Self> {
        let runtime = runtime.take().unwrap_or_else(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
        });
        user_options.client_options.connect_timeout =
            login_timeout.map(|to| Duration::new(to as u64, 0));
        let guard = runtime.enter();
        let client = runtime.block_on(async {
            Client::with_options(user_options.client_options).map_err(Error::InvalidClientOptions)
        })?;
        drop(guard);
        let uuid_repr = user_options.uuid_representation;
        let connection = MongoConnection {
            client,
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
            uuid_repr,
            runtime,
        };
        // Verify that the connection is working and the user has access to the default DB
        // ADF is supposed to check permissions on this
        MongoQuery::prepare(&connection, current_db, None, "select 1", type_mode)?;
        Ok(connection)
    }

    pub fn shutdown(self) -> Result<()> {
        self.runtime
            .block_on(async { self.client.shutdown().await });
        drop(self.runtime);
        Ok(())
    }

    /// Gets the ADF version the client is connected to.
    pub fn get_adf_version(&self) -> Result<String> {
        let guard = self.runtime.enter();
        let res = self.runtime.block_on(async {
            let db = self.client.database("admin");
            let cmd_res = db
                .run_command(doc! {"buildInfo": 1}, None)
                .await
                .map_err(Error::DatabaseVersionRetreival)?;
            let build_info: BuildInfoResult =
                bson::from_document(cmd_res).map_err(Error::DatabaseVersionDeserialization)?;
            Ok(build_info.data_lake.version)
        });
        drop(guard);
        res
    }

    /// cancels all queries for a given statement id
    pub fn cancel_queries_for_statement(&self, statement_id: Bson) -> Result<bool> {
        let _guard = self.runtime.enter();
        self.runtime.handle().block_on(async {
            // use $currentOp and match the comment field to identify any queries issued by the current statement
            let current_ops_pipeline = vec![
                doc! {"$currentOp": {}},
                doc! {"$match": {"command.comment": statement_id}},
            ];
            let admin_db = self.client.database("admin");
            let mut cursor = admin_db
                .aggregate(current_ops_pipeline, None)
                .await
                .map_err(Error::QueryExecutionFailed)?;

            // iterate through the results and kill the operations
            while cursor.advance().await.map_err(Error::QueryCursorUpdate)? {
                let operation = cursor
                    .deserialize_current()
                    .map_err(Error::QueryCursorUpdate)?;
                if let Some(operation_id) = operation.get("opid") {
                    let killop_doc = doc! { "killOp": 1, "op": operation_id};
                    admin_db
                        .run_command(killop_doc, None)
                        .await
                        .map_err(Error::QueryExecutionFailed)?;
                }
            }
            Ok(true)
        })
    }
}

// Struct representing the response for a buildInfo command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct BuildInfoResult {
    pub ok: i32,
    pub version: String,
    #[serde(rename = "versionArray")]
    pub version_array: Vec<i32>,
    #[serde(rename = "dataLake")]
    pub data_lake: DataLakeBuildInfo,
}

// Auxiliary struct representing part of the response for a buildInfo command.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct DataLakeBuildInfo {
    pub version: String,
    #[serde(rename = "gitVersion")]
    pub git_version: String,
    pub date: String,
}
