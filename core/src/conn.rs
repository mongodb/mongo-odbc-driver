use crate::error::Error;
use mongodb::sync::Client;

#[derive(Debug)]
pub struct MongoConnection {
    // The mongo DB client
    pub client: Client,
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
    ) -> Result<Self, Error> {
        unimplemented!()
    }
}
