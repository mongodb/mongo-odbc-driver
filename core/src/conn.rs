use crate::MongoQuery;
use crate::{err::Result, Error};
use bson::doc;
use mongodb::{options::ClientOptions, sync::Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoConnection {
    /// The mongo DB client
    pub client: Client,
    /// The current database set for this client.
    /// All new queries will be done on this DB.
    /// We stick with mongo terminology here and ODBC
    /// terminology in the odbc wrappers, hence
    /// current_db here and current_catalog in the
    /// odbc/handles code.
    pub current_db: Option<String>,
    /// Number of seconds to wait for any request on the connection to complete before returning to
    /// the application.
    /// Comes from SQL_ATTR_CONNECTION_TIMEOUT if set. Used any time there is a time out in a
    /// situation not associated with query execution or login.
    pub operation_timeout: Option<Duration>,
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
        mut client_options: ClientOptions,
        current_db: Option<&str>,
        operation_timeout: Option<u32>,
        login_timeout: Option<u32>,
    ) -> Result<Self> {
        client_options.connect_timeout = login_timeout.map(|to| Duration::new(to as u64, 0));
        let client = Client::with_options(client_options).map_err(Error::InvalidClientOptions)?;
        let connection = MongoConnection {
            client,
            current_db: current_db.map(String::from),
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
        };
        // Verify that the connection is working and the user has access to the default DB
        MongoQuery::execute(&connection, None, "select 1")?;
        Ok(connection)
    }

    /// Gets the ADF version the client is connected to.
    pub fn get_adf_version(&self) -> Result<String> {
        let db = self.client.database("admin");
        let cmd_res = db
            .run_command(doc! {"buildInfo": 1}, None)
            .map_err(Error::DatabaseVersionRetreival)?;
        let build_info: BuildInfoResult =
            bson::from_document(cmd_res).map_err(Error::DatabaseVersionDeserialization)?;
        Ok(build_info.data_lake.version)
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
