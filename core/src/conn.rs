use crate::odbc_uri::UserOptions;
use crate::{err::Result, Error};
use crate::{MongoQuery, TypeMode};
use lazy_static::lazy_static;
use mongodb::{
    bson::{doc, Bson, UuidRepresentation},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Weak},
    time::Duration,
};

// we make from UserOptions to Weak<Client> so that we do not hold around
// clients that are no longer in use. In most cases it won't matter, but for drivers that live in
// memory for a long time, this could be a problem.
struct ClientMap(Vec<(UserOptions, Weak<Client>)>);

impl ClientMap {
    // get returns the client associated with the given user options if it exists.
    fn get(&mut self, user_options: &UserOptions) -> Option<Arc<Client>> {
        let mut remover = None;
        for (i, (options, weak_client)) in self.0.iter().enumerate() {
            if options == user_options {
                if let Some(client) = weak_client.upgrade() {
                    return Some(client);
                // if somehow the client cannot be upgraded, we want to remove this entry in the
                // ClientMap.
                } else {
                    remover = Some(i);
                }
            }
        }
        // We have to do this this way due to the borrow checker, it would be nice if we could just
        // remove in the else above, but we cannot.
        if let Some(i) = remover {
            self.0.remove(i);
        }
        None
    }

    // gc is garbage collection of any clients that are no longer in use.
    fn gc(&mut self) {
        self.0
            .retain(|(_, weak_client)| std::sync::Weak::strong_count(weak_client) == 0);
    }

    // insert inserts a new client into the client map keyed on the user options.
    fn insert(&mut self, user_options: UserOptions, client: &Arc<Client>) {
        let client = std::sync::Arc::downgrade(client);
        self.0.push((user_options, client));
    }

    // new creates a new ClientMap.
    fn new() -> Self {
        ClientMap(Vec::new())
    }
}

lazy_static! {
    static ref CLIENT_MAP: std::sync::Mutex<ClientMap> = std::sync::Mutex::new(ClientMap::new());
}

#[derive(Debug)]
#[repr(C)]
pub struct MongoConnection {
    /// The mongo DB client
    pub client: Arc<Client>,
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
        let mut client_map = CLIENT_MAP.lock().unwrap();
        let client = if let Some(client) = client_map.get(&user_options) {
            client
        } else {
            let key_user_options = user_options.clone();
            let client = runtime.block_on(async {
                Client::with_options(user_options.client_options)
                    .map_err(Error::InvalidClientOptions)
            })?;
            let client = Arc::new(client);
            client_map.insert(key_user_options, &client);
            client
        };
        // keep in mind, mutexes should always be dropped in reverse order of acquisition to avoid
        // possible deadlocks
        drop(client_map);
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
        //        // we need to lock the CLIENT_MAP to potentially remove the client from the map.
        //        // This prevents races on the strong_count or with gc().
        //        let mut client_map = CLIENT_MAP.lock().unwrap();
        //        if let Some(client) = std::sync::Arc::into_inner(self.client) {
        //            self.runtime.block_on(async { client.shutdown().await });
        //        }
        //        // garbage collect any clients that are no longer in use. It's possible there could be
        //        // other clients that are no longer in use because shutdown was not properly called before,
        //        // so we gc them all.
        //        client_map.gc();
        //        drop(self.runtime);
        Ok(())
    }

    /// Gets the ADF version the client is connected to.
    pub fn get_adf_version(&self) -> Result<String> {
        let _guard = self.runtime.enter();
        self.runtime.block_on(async {
            let db = self.client.database("admin");
            let cmd_res = db
                .run_command(doc! {"buildInfo": 1})
                .await
                .map_err(Error::DatabaseVersionRetreival)?;
            let build_info: BuildInfoResult = mongodb::bson::from_document(cmd_res)
                .map_err(Error::DatabaseVersionDeserialization)?;
            Ok(build_info.data_lake.version)
        })
    }

    /// cancels all queries for a given statement id
    pub fn cancel_queries_for_statement(&self, statement_id: Bson) -> Result<bool> {
        let _guard = self.runtime.enter();
        // because there are so many awaits in this function, the bulk of the function is wrapped in a block_on
        self.runtime.block_on(async {
            // use $currentOp and match the comment field to identify any queries issued by the current statement
            let current_ops_pipeline = vec![
                doc! {"$currentOp": {}},
                doc! {"$match": {"command.comment": statement_id}},
            ];
            let admin_db = self.client.database("admin");
            let mut cursor = admin_db
                .aggregate(current_ops_pipeline)
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
                        .run_command(killop_doc)
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
