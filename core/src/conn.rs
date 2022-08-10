use crate::err::Result;
use bson::doc;
use mongodb::{options::ClientOptions, sync::Client};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoConnection {
    // The mongo DB client
    pub client: Client,
    // The current database set for this client.
    // All new queries will be done on this DB.
    // We stick with mongo terminalogy here and ODBC
    // terminalogy in the odbc wrappers, hence
    // current_db here and current_catalog in the
    // odbc/handles code.
    pub current_db: Option<String>,
    // Number of seconds to wait for any request on the connection to complete before returning to
    // the application.
    // Comes from SQL_ATTR_CONNECTION_TIMEOUT if set. Used any time there is a time out in a
    // situation not associated with query execution or login.
    pub operation_timeout: Option<Duration>,
}

const MONGODB_ODBC_DRIVER: &str = "mongo_odbc_driver";

impl MongoConnection {
    // Creates a new MongoConnection with the given settings and lists databases to make sure the
    // connection is legit.
    //
    // The operation will timeout if it takes more than loginTimeout seconds. This timeout is
    // delegated to the mongo rust driver.
    //
    // The initial current database if provided should come from SQL_ATTR_CURRENT_CATALOG
    // and will take precedence over the database setting specified in the uri if any.
    // The initial operation time if provided should come from and will take precedence over the
    // setting specified in the uri if any.
    pub fn connect(
        mongo_uri: &str,
        auth_src: &str,
        current_db: Option<&str>,
        operation_timeout: Option<u32>,
        login_timeout: Option<u32>,
        application_name: Option<&str>,
    ) -> Result<Self> {
        let mut client_options = ClientOptions::parse(mongo_uri)?;
        client_options.connect_timeout = login_timeout.map(|to| Duration::new(to as u64, 0));
        // set application name, note that users can set their own application name, or we default
        // to mongo-odbc-driver.
        client_options.app_name = application_name
            .map(String::from)
            .or_else(|| Some(MONGODB_ODBC_DRIVER.to_string()));
        let client = Client::with_options(client_options)?;
        // run the "ping" command on the `auth_src` database. We assume this requires the
        // fewest permissions of anything we can do to verify a connection.
        client
            .database(auth_src)
            .run_command(doc! {"ping": 1}, None)?;
        Ok(MongoConnection {
            client,
            current_db: current_db.map(String::from),
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
        })
    }
}
