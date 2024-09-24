use crate::cluster_type::{determine_cluster_type, MongoClusterType};
use crate::load_library::{get_mongosqltranslate_library, load_mongosqltranslate_library};
use crate::odbc_uri::UserOptions;
use crate::{err::Result, Error};
use crate::{MongoQuery, TypeMode};
use bson::Document;
use constants::DRIVER_ODBC_VERSION;
use definitions::LibmongosqltranslateCommand;
use lazy_static::lazy_static;
use libloading::Symbol;
use mongodb::{
    bson::{doc, Bson, UuidRepresentation},
    Client,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "garbage_collect")]
use std::sync::Weak;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Runtime;

// we make from UserOptions to Client and Weak<Runtime> so that we do not hold around
// Clients and Runtimes that are no longer in use. In most cases it won't matter, but for drivers that live in
// memory for a long time, this could be a problem.
#[cfg(feature = "garbage_collect")]
struct ClientMap(Vec<(UserOptions, Client, Weak<Runtime>)>);
#[cfg(not(feature = "garbage_collect"))]
struct ClientMap(Vec<(UserOptions, Client, Arc<Runtime>)>);

impl ClientMap {
    // new creates a new ClientMap.
    fn new() -> Self {
        ClientMap(Vec::new())
    }

    // get returns the client associated with the given user options if it exists.
    #[cfg(feature = "garbage_collect")]
    fn get(&mut self, user_options: &UserOptions) -> Option<(Client, Arc<Runtime>)> {
        let mut to_remove = Vec::new();
        for (i, (options, client, weak_rt)) in self.0.iter().enumerate() {
            if options == user_options {
                if let Some(rt) = weak_rt.upgrade() {
                    return Some((client.clone(), rt));
                // if somehow the Runtime cannot be upgraded, we want to remove this entry in the
                // ClientMap, because the Runtime associated with the Client has been dropped,
                // which will make the Topology hang forever.
                } else {
                    to_remove.push(i);
                }
            }
        }
        // We have to do this this way due to the borrow checker, it would be nice if we could just
        // remove in the else above, but we cannot.
        for i in to_remove.into_iter().rev() {
            self.0.remove(i);
        }
        None
    }

    // get returns the client associated with the given user options if it exists.
    #[cfg(not(feature = "garbage_collect"))]
    fn get(&mut self, user_options: &UserOptions) -> Option<(Client, Arc<Runtime>)> {
        for (options, client, rt) in self.0.iter() {
            if options == user_options {
                return Some((client.clone(), rt.clone()));
            }
        }
        None
    }

    // gc is garbage collection of any clients that are no longer in use.
    #[cfg(feature = "garbage_collect")]
    fn gc(&mut self) {
        self.0
            .retain(|(_, _, weak_client)| Weak::strong_count(weak_client) != 0);
    }

    // insert inserts a new client into the client map keyed on the user options. Note this will
    // insert duplicates and it is on the user to check if an entry already exists.
    #[cfg(feature = "garbage_collect")]
    fn insert(&mut self, user_options: UserOptions, client: &Client, rt: &Arc<Runtime>) {
        self.0
            .push((user_options, client.clone(), Arc::downgrade(rt)));
    }

    #[cfg(not(feature = "garbage_collect"))]
    fn insert(&mut self, user_options: UserOptions, client: &Client, rt: &Arc<Runtime>) {
        self.0.push((user_options, client.clone(), rt.clone()));
    }
}

lazy_static! {
    static ref CLIENT_MAP: tokio::sync::Mutex<ClientMap> =
        tokio::sync::Mutex::new(ClientMap::new());
}

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
    pub runtime: Arc<Runtime>,

    /// client cluster type. Valid types are AtlasDataFederation and Enterprise
    pub cluster_type: MongoClusterType,
}

impl MongoConnection {
    fn get_client_and_runtime(
        user_options: UserOptions,
        runtime: Arc<Runtime>,
    ) -> Result<(Client, Arc<Runtime>)> {
        let mut client_map = runtime.block_on(async { CLIENT_MAP.lock().await });
        if let Some(cv) = client_map.get(&user_options) {
            log::info!("reusing Client");
            Ok(cv)
        } else {
            let key_user_options = user_options.clone();
            log::info!("creating new Client",);
            let guard = runtime.enter();
            // the Client Topology uses tokio::spawn, so we need a guard here.
            let client = runtime.block_on(async {
                Client::with_options(user_options.client_options)
                    .map_err(Error::InvalidClientOptions)
            })?;
            // we need to drop the guard before we return the runtime to kill the borrow
            // on the runtime. We drop it before the insert to hold the lock for as little time as
            // possible.
            drop(guard);
            client_map.insert(key_user_options, &client, &runtime);
            Ok((client, runtime))
        }
    }

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
        mut runtime: Option<Runtime>,
        max_string_length: Option<u16>,
    ) -> Result<Self> {
        let runtime = Arc::new(runtime.take().unwrap_or_else(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
        }));
        user_options.client_options.connect_timeout =
            login_timeout.map(|to| Duration::new(u64::from(to), 0));
        let uuid_repr = user_options.uuid_representation;
        let (client, runtime) = Self::get_client_and_runtime(user_options, runtime)?;

        load_mongosqltranslate_library();

        if let Some(library) = get_mongosqltranslate_library() {
            // get runCommand
            let run_command_function: Symbol<
                'static,
                unsafe extern "C" fn(LibmongosqltranslateCommand) -> LibmongosqltranslateCommand,
            > = unsafe {
                library
                    .get(b"runCommand")
                    .expect("Failed to load runCommand symbol")
            };

            // getLibraryVersion
            let get_library_version_command = doc! {
                "command": "getMongosqlTranslateVersion",
                "options": {},
            };

            let get_library_version_command_bytes = bson::to_vec(&get_library_version_command)
                .expect("Failed to serialize Document into BSON bytes");

            let length = get_library_version_command_bytes.len();

            let capacity = get_library_version_command_bytes.capacity();

            let get_library_version_mongosqltranslate_command = LibmongosqltranslateCommand {
                data: Box::into_raw(get_library_version_command_bytes.into_boxed_slice()).cast(),
                length,
                capacity,
            };

            let decomposed_library_version =
                unsafe { run_command_function(get_library_version_mongosqltranslate_command) };

            let returned_doc: Document = unsafe {
                bson::from_slice(
                    Vec::from_raw_parts(
                        decomposed_library_version.data.cast_mut(),
                        decomposed_library_version.length,
                        decomposed_library_version.capacity,
                    )
                    .as_slice(),
                )
                .expect("Failed to deserialize result")
            };

            let libmongosql_library_version = returned_doc
                .get("version")
                .expect("`version` was missing")
                .as_str()
                .expect("`version` should be a String");

            dbg!(libmongosql_library_version);

            // do something with library version
            // where do I put the library version for the logs?

            // CheckDriverVersion
            let check_driver_version_command = doc! {
                "command": "checkDriverVersion",
                "options": {
                    "driverVersion": DRIVER_ODBC_VERSION.clone(),
                    "odbcDriver": true
                },
            };

            let check_driver_version_command_bytes = bson::to_vec(&check_driver_version_command)
                .expect("Failed to serialize Document into BSON bytes");

            let length = check_driver_version_command_bytes.len();

            let capacity = check_driver_version_command_bytes.capacity();

            let check_driver_version_mongosqltranslate_command = LibmongosqltranslateCommand {
                data: Box::into_raw(check_driver_version_command_bytes.into_boxed_slice()).cast(),
                length,
                capacity,
            };

            let decomposed_library_compatibility =
                unsafe { run_command_function(check_driver_version_mongosqltranslate_command) };

            let returned_doc: Document = unsafe {
                bson::from_slice(
                    Vec::from_raw_parts(
                        decomposed_library_compatibility.data.cast_mut(),
                        decomposed_library_compatibility.length,
                        decomposed_library_compatibility.capacity,
                    )
                    .as_slice(),
                )
                .expect("Failed to deserialize result")
            };

            let is_libmongosql_library_compatible = returned_doc
                .get("compatibility")
                .expect("`compatibility` was missing")
                .as_bool()
                .expect("`compatibility` should be a bool");

            if !is_libmongosql_library_compatible {
                return Err(Error::LibmongosqltranslateLibraryIsIncompatible(
                    &DRIVER_ODBC_VERSION,
                ));
            }
        }

        let type_of_cluster = runtime.block_on(async { determine_cluster_type(&client).await });
        match type_of_cluster {
            MongoClusterType::AtlasDataFederation => {}
            MongoClusterType::Community => {
                // Community edition is not supported
                return Err(Error::UnsupportedClusterConfiguration(
                    "Community edition detected. The driver is intended for use with MongoDB Enterprise edition or Atlas Data Federation.".to_string(),
                ));
            }
            MongoClusterType::Enterprise => {
                // Ensure the library is loaded if Enterprise edition is detected
                if get_mongosqltranslate_library().is_none() {
                    return Err(Error::UnsupportedClusterConfiguration(
                        "Enterprise edition detected, but mongosqltranslate library not found."
                            .to_string(),
                    ));
                }
            }
            MongoClusterType::UnknownTarget => {
                // Unknown cluster type is not supported
                return Err(Error::UnsupportedClusterConfiguration(
                    "Unknown cluster/target type detected. The driver is intended for use with MongoDB Enterprise edition or Atlas Data Federation.".to_string(),
                ));
            }
        }

        let connection = MongoConnection {
            client,
            operation_timeout: operation_timeout.map(|to| Duration::new(u64::from(to), 0)),
            uuid_repr,
            runtime,
            cluster_type: type_of_cluster,
        };

        // Verify that the connection is working and the user has access to the default DB
        // ADF is supposed to check permissions on this
        MongoQuery::prepare(
            &connection,
            current_db,
            None,
            "select 1",
            type_mode,
            max_string_length,
        )?;

        Ok(connection)
    }

    #[cfg(feature = "garbage_collect")]
    pub fn shutdown(self) -> Result<()> {
        // we need to lock the CLIENT_MAP to potentially remove the client from the map.
        // This prevents races on the strong_count or with gc().
        let mut client_map = self.runtime.block_on(async { CLIENT_MAP.lock().await });
        // If this is the last reference to the Runtime, we need to shutdown the Client.
        if Arc::strong_count(&self.runtime) == 1 {
            self.runtime
                .block_on(async { self.client.shutdown().await });
        }
        // We need to drop the Runtime before we gc, otherwise this will not be collected.
        drop(self.runtime);
        // garbage collect any Clients and Runtimes that are no longer in use, which is denoted by
        // the strong_count for the Runtime being 0. It's possible there could be
        // other clients that are no longer in use because shutdown was not properly called before,
        // so we gc them all.
        client_map.gc();
        Ok(())
    }

    #[cfg(not(feature = "garbage_collect"))]
    pub fn shutdown(self) -> Result<()> {
        if Arc::strong_count(&self.runtime) == 1 {
            self.runtime
                .block_on(async { self.client.shutdown().await });
        }
        Ok(())
    }

    /// Gets the ADF version the client is connected to.
    pub fn get_adf_version(&self) -> Result<String> {
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
