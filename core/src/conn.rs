use crate::err::Result;
use mongodb::{options::ClientOptions, sync::Client};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoConnection {
    // The mongo DB client
    pub client: Client,
    // The current database set for this client.
    // All new queries will be done on this DB.
    pub current_db: Option<String>,
    // Number of seconds to wait for any request on the connection to complete before returning to
    // the application.
    // Comes from SQL_ATTR_CONNECTION_TIMEOUT if set. Used any time there is a time out in a
    // situation not associated with query execution or login.
    pub operation_timeout: Option<Duration>,
}

impl MongoConnection {
    // Creates a new MongoConnection with the given settings and execute a dummy aggregation to
    // validate it works.
    // The operation will timeout if it takes more than loginTimeout seconds.
    // The initial current database if provided should come from SQL_ATTR_CURRENT_CATALOG
    // and will take precedence over the database setting specified in the uri if any.
    // The initial operation time if provided should come from and will take precedence over the
    // setting specified in the uri if any.
    pub fn connect(
        uri: &str,
        current_db: Option<String>,
        operation_timeout: Option<i32>,
        login_timeout: Option<i32>,
    ) -> Result<Self> {
        println!("uri = {:?}", uri);
        // for now, assume we get a mongodb uri
        let client_options = ClientOptions::parse(uri)?;
        // set application name?
        let client = Client::with_options(client_options)?;
        Ok(MongoConnection {
            client,
            current_db,
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
        })
    }
}
