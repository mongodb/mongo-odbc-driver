use mongodb::sync::Client;
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub struct MongoConnection {
    // The mongo DB client
    pub client: Client,
    // The current database set for this client.
    // All new queries will be done on this DB.
    pub current_db: String,
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
    // The initial operation time if provided should come from  and will take precedence over the
    // setting specified in the uri if any.
    pub fn connect(
        uri: &str,
        current_db: Option<&str>,
        operation_timeout: Option<i32>,
        login_timeout: Option<i32>,
    ) -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }
}
