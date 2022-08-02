use crate::err::{Error, Result};
use mongodb::{options::ClientOptions, sync::Client};
use std::{collections::BTreeMap, time::Duration};

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
        uri: &str,
        current_db: Option<String>,
        operation_timeout: Option<i32>,
        login_timeout: Option<i32>,
    ) -> Result<Self> {
        println!("CONNNECEJFLKDSJFSD");
        let mut attributes = MongoConnection::get_attributes(uri)?;
        let current_db = if current_db.is_none() {
            attributes.remove("database")
        } else {
            current_db
        };
        let (user, attributes) = MongoConnection::get_attribute(attributes, "user")?;
        let (pwd, attributes) = MongoConnection::get_attribute(attributes, "pwd")?;
        let (server, mut attributes) = MongoConnection::get_attribute(attributes, "server")?;
        let auth_src = attributes
            .remove("auth_src")
            .unwrap_or_else(|| "admin".to_string());

        let mongo_uri = format!(
            "mongodb://{}:{}@{}/{}?ssl=true",
            user, pwd, server, auth_src
        );
        println!("MONGO URI: {}", mongo_uri);

        // for now, assume server attribute is a mongodb uri
        let client_options = ClientOptions::parse(mongo_uri);
        let mut client_options = client_options?;
        client_options.connect_timeout = login_timeout.map(|to| Duration::new(to as u64, 0));
        // set application name?
        let client = Client::with_options(client_options)?;
        // list databases to check connection
        let _ = client.list_database_names(None, None)?;
        Ok(MongoConnection {
            client,
            current_db,
            operation_timeout: operation_timeout.map(|to| Duration::new(to as u64, 0)),
        })
    }

    fn get_attribute(
        mut attributes: BTreeMap<String, String>,
        name: &str,
    ) -> Result<(String, BTreeMap<String, String>)> {
        let ret = attributes.remove(name);
        if let Some(ret) = ret {
            Ok((ret, attributes))
        } else {
            let err_string = format!("uri must contain '{}' attribute", name);
            Err(Error::UriFormatError(err_string))
        }
    }

    fn get_attributes(uri: &str) -> Result<BTreeMap<String, String>> {
        if uri.is_empty() {
            return Err(Error::UriFormatError("uri must not be empty".to_string()));
        }
        // TODO: handle DSN expansion here (i.e., lookup the DSN and add the resolved attributes).
        // split the uri attributes on ';'
        uri.split(';')
            .map(|attr| {
                // now split each attribute pair on '='
                let mut sp = attr.split('=').map(String::from).collect::<Vec<_>>();
                if sp.len() != 2 {
                    return Err(Error::UriFormatError(
                        "all uri atttributes must be of the form key=value".to_string(),
                    ));
                }
                // ODBC attribute keys are case insensitive, so we lowercase the keys
                Ok((
                    // to_lowercase creates a String since it copies bytes
                    sp.remove(0).trim().to_lowercase(),
                    // trim just returns pointers into the original String/str, so we need
                    // to_string
                    sp.remove(0).trim().to_string(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()
    }
}

#[test]
fn test_get_attributes() {
    use crate::map;

    assert!(MongoConnection::get_attributes("").is_err());
    assert!(MongoConnection::get_attributes("Foo").is_err());
    assert!(MongoConnection::get_attributes("driver=Foo;Bar").is_err());

    let expected: BTreeMap<String, String> = map! {"driver".to_string() => "Foo".to_string()};
    assert_eq!(
        expected,
        MongoConnection::get_attributes("Driver=Foo").unwrap()
    );

    let expected: BTreeMap<String, String> =
        map! {"driver".to_string() => "Foo".to_string(), "server".to_string() => "bAr".to_string()};
    assert_eq!(
        expected,
        MongoConnection::get_attributes("Driver=Foo;SERVER=bAr").unwrap()
    );

    assert_eq!(
        expected,
        MongoConnection::get_attributes("    Driver  =  Foo   ;    SERVER  =   bAr  ").unwrap()
    );
}
