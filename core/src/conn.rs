use mongodb::sync::Client;
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub struct MongoConnection {
    // The mongo DB client
    pub client: Client,
    // The current database set for this client.
    // All new queries will be done on this DB.
    pub current_db: Box<str>,
    // Number of seconds to wait for any request on the connection to complete before returning to
    // the application.
    // Comes from SQL_ATTR_CONNECTION_TIMEOUT if set. Used for database metadata.
    pub operation_timeout: Option<Duration>,
}

impl MongoConnection {
    // Creates a new MongoConnection with the given settings and execute a dummy aggregation to
    // validate it works.
    // The operation will timeout if it takes more than loginTimeout seconds.
    // The current database if provided should come from SQL_ATTR_CURRENT_CATALOG
    // and will take precedence over the database setting specified in the uri if any.
    pub fn connect(
        uri: &str,
        current_db: Option<&str>,
        login_time_out: Option<i32>,
    ) -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }
}
