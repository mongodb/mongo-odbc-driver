use crate::odbc_uri::UserOptions;
use crate::MongoQuery;
use crate::{err::Result, Error};
use bson::{doc, UuidRepresentation};
use mongodb::sync::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
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
    ) -> Result<Self> {
        user_options.client_options.connect_timeout =
            login_timeout.map(|to| Duration::new(to as u64, 0));
        let client = Client::with_options(user_options.client_options)
            .map_err(Error::InvalidClientOptions)?;
        let uuid_repr = user_options.uuid_representation;
        let connection = MongoConnection {
            client,
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
            uuid_repr,
        };
        // Verify that the connection is working and the user has access to the default DB
        MongoQuery::execute(&connection, current_db, None, "select 1")?;
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
