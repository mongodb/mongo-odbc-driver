use crate::err::{Error, Result};
use constants::{DEFAULT_APP_NAME, DRIVER_SHORT_NAME};
use lazy_static::lazy_static;
use mongodb::{
    bson::UuidRepresentation,
    options::{
        AuthMechanism, ClientOptions, ConnectionString, Credential, DriverInfo, ResolverConfig,
        ServerAddress,
    },
};
use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use shared_sql_utils::Dsn;
use std::collections::HashMap;
use std::str::FromStr;

const EMPTY_URI_ERROR: &str = "URI must not be empty";
const INVALID_ATTR_FORMAT_ERROR: &str = "all URI attributes must be of the form keyword=value";
const MISSING_CLOSING_BRACE_ERROR: &str = "attribute value beginning with '{' must end with '}'";

pub const DATABASE: &str = "database";
pub const DRIVER: &str = "driver";
pub const DSN: &str = "dsn";
pub const PASSWORD: &str = "password";
pub const PWD: &str = "pwd";
pub const SERVER: &str = "server";
pub const USER: &str = "user";
pub const UID: &str = "uid";
pub const URI: &str = "uri";
pub const APPNAME: &str = "appname";
pub const LOGLEVEL: &str = "loglevel";
pub const SIMPLE_TYPES_ONLY: &str = "simple_types_only";
pub const ENABLE_MAX_STRING_LENGTH: &str = "enable_max_string_length";

const POWERBI_CONNECTOR: &str = "powerbi-connector";

const URI_KWS: &[&str] = &[URI];
const USER_KWS: &[&str] = &[UID, USER];
const PWD_KWS: &[&str] = &[PASSWORD, PWD];
const SERVER_KWS: &[&str] = &[SERVER];

lazy_static! {
    static ref KEYWORDS: RegexSet = RegexSetBuilder::new(
        [
            DATABASE,
            DRIVER,
            DSN,
            PASSWORD,
            PWD,
            SERVER,
            USER,
            UID,
            URI,
            APPNAME,
            LOGLEVEL,
            SIMPLE_TYPES_ONLY,
            ENABLE_MAX_STRING_LENGTH,
        ]
        .into_iter()
        .map(|x| "^".to_string() + x + "$")
        .collect::<Vec<_>>()
    )
    .case_insensitive(true)
    .build()
    .unwrap();
    static ref AUTH_SOURCE_REGEX: Regex = RegexBuilder::new(r#"[&?]authSource=(?P<source>[^&]*)"#)
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref AUTH_MECHANISM_REGEX: Regex =
        RegexBuilder::new(r#"[&?]authMechanism=(?P<mechanism>[^&]*)"#)
            .case_insensitive(true)
            .build()
            .unwrap();
    static ref USERNAME_PASSWORD_REGEX: Regex = Regex::new(r#"(^.*)@.*/?(.*)?"#).unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserOptions {
    pub client_options: ClientOptions,
    pub uuid_representation: Option<UuidRepresentation>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ODBCUri(HashMap<String, String>);

impl std::ops::Deref for ODBCUri {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn transform_keyword(keyword: &str) -> String {
    match keyword {
        UID => USER.to_string(),
        USER => USER.to_string(),
        PWD => PASSWORD.to_string(),
        PASSWORD => PASSWORD.to_string(),
        _ => keyword.to_string(),
    }
}

impl ODBCUri {
    pub fn new(odbc_uri: String) -> Result<ODBCUri> {
        if odbc_uri.is_empty() {
            return Err(Error::InvalidUriFormat(EMPTY_URI_ERROR.to_string()));
        }
        let mut ret = ODBCUri::process_uri(odbc_uri.clone())?;
        if ret.get(DSN).is_some() {
            let mut dsn_opts = Dsn::from_attribute_string(&odbc_uri);
            dsn_opts = dsn_opts.from_private_profile_string().unwrap();
            ret = ODBCUri::process_uri(format!("{odbc_uri};{}", dsn_opts.to_connection_string()))?;
        }
        Ok(ret)
    }

    fn process_uri(odbc_uri: String) -> Result<ODBCUri> {
        let mut input = odbc_uri;
        let mut ret = ODBCUri(HashMap::new());
        while let Some((keyword, value, rest)) = ODBCUri::get_next_attribute(input)? {
            // if attributes are repeated, the first is the one that is kept.

            ret.0.entry(transform_keyword(&keyword)).or_insert(value);
            if rest.is_none() {
                break;
            }
            input = rest.unwrap();
        }
        Ok(ret)
    }

    fn get_next_attribute(odbc_uri: String) -> Result<Option<(String, String, Option<String>)>> {
        // clean up any extra semi-colons
        let index = odbc_uri.find(|c| c != ';');
        // these are just trailing semis on the URI
        if index.is_none() {
            return Ok(None);
        }
        let odbc_uri = odbc_uri.get(index.unwrap()..).unwrap();
        // find the first '=' sign, '=' does not appear in any keywords, so this is safe.
        let (keyword, rest) = odbc_uri.split_at(
            odbc_uri
                .find('=')
                .ok_or_else(|| Error::InvalidUriFormat(INVALID_ATTR_FORMAT_ERROR.to_string()))?,
        );
        // remove the leading '=' sign.
        let rest = rest.get(1..).unwrap();
        if !KEYWORDS.is_match(keyword) {
            return Err(Error::InvalidUriFormat(format!(
                "'{keyword}' is not a valid URI keyword"
            )));
        }
        let (value, rest) = if rest.starts_with('{') {
            let rest = rest
                .get(1..)
                .ok_or_else(|| Error::InvalidUriFormat(MISSING_CLOSING_BRACE_ERROR.to_string()))?;
            ODBCUri::handle_braced_value(rest)?
        } else {
            ODBCUri::handle_unbraced_value(rest)?
        };
        Ok(Some((keyword.to_lowercase(), value, rest)))
    }

    fn handle_braced_value(input: &str) -> Result<(String, Option<String>)> {
        let mut after_brace = false;
        // This is a simple two state state machine. Either the previous character was '}'
        // or it is not. When the previous character was '}' and we have reached the end
        // of the input or the current character is ';', we have found the entire value for
        // this attribute.
        for (i, c) in input.char_indices() {
            if after_brace && c == ';' {
                let mut rest = input.get(i + 1..);
                if rest.unwrap() == "" {
                    rest = None;
                }
                return Ok((
                    input.get(0..i - 1).unwrap().to_string(),
                    rest.map(String::from),
                ));
            }
            if c == '}' {
                if i + 1 == input.len() {
                    return Ok((input.get(0..i).unwrap().to_string(), None));
                }
                after_brace = true
            } else {
                after_brace = false
            }
        }
        Err(Error::InvalidUriFormat(
            MISSING_CLOSING_BRACE_ERROR.to_string(),
        ))
    }

    fn handle_unbraced_value(input: &str) -> Result<(String, Option<String>)> {
        let index = input.find(';');
        if index.is_none() {
            return Ok((input.to_string(), None));
        }
        let (value, rest) = input.split_at(index.unwrap());
        if rest.len() == 1 {
            return Ok((value.to_string(), None));
        }
        Ok((value.to_string(), rest.get(1..).map(String::from)))
    }

    // get_attribute will return the first value with a given one of the names passed, assuming all names,
    // without modifying internal state.
    pub fn get_attribute(&self, names: &[&str]) -> Option<&String> {
        for name in names.iter() {
            if let Some(value) = self.0.get(*name) {
                return Some(value);
            }
        }
        None
    }

    // remove will remove the first value with a given one of the names passed, assuming all names
    // are synonyms.
    pub fn remove(&mut self, names: &[&str]) -> Option<String> {
        for name in names.iter() {
            let ret = self.0.remove(*name);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }

    // remove_or_else is the same as remove but with a default thunk.
    pub fn remove_or_else(&mut self, f: impl Fn() -> String, names: &[&str]) -> String {
        self.remove(names).unwrap_or_else(f)
    }

    // remove_mandatory_attribute will find an attribute that must exist and transfer ownership to
    // the caller. It accepts a slice of names that will be checked in order for names that are
    // synonyms (e.g., uid and user are both viable attribute names for a user). If both names
    // exist, it will only find the first.
    fn remove_mandatory_attribute(&mut self, names: &[&str]) -> Result<String> {
        self.remove(names).ok_or_else(|| {
            if names.len() == 1 {
                Error::InvalidUriFormat(format!(
                    "{} is required for a valid Mongo ODBC Uri",
                    names[0]
                ))
            } else {
                Error::InvalidUriFormat(format!(
                    "One of {names:?} is required for a valid Mongo ODBC Uri"
                ))
            }
        })
    }

    // try_into_client_options converts this ODBCUri to a mongo_uri String. It will
    // remove all the attributes necessary to make a mongo_uri. This is destructive!
    pub async fn try_into_client_options(&mut self) -> Result<UserOptions> {
        let uri = self.remove(URI_KWS);
        if let Some(uri) = uri {
            return self.handle_uri(&uri).await;
        }
        self.handle_no_uri()
    }

    fn check_client_opts_credentials(client_options: &ClientOptions) -> Result<()> {
        if client_options
            .credential
            .as_ref()
            .unwrap()
            .username
            .is_none()
        {
            return Err(Error::InvalidUriFormat(format!(
                "One of {USER_KWS:?} is required for a valid Mongo ODBC Uri"
            )));
        }
        if client_options
            .credential
            .as_ref()
            .unwrap()
            .password
            .is_none()
        {
            return Err(Error::InvalidUriFormat(format!(
                "One of {PWD_KWS:?} is required for a valid Mongo ODBC Uri"
            )));
        }
        Ok(())
    }

    fn set_server_and_source(
        opts: &mut ClientOptions,
        server: Option<String>,
        source: Option<String>,
    ) -> Result<()> {
        // server should supercede that specified in the uri, if specified.
        if let Some(server) = server {
            opts.hosts = vec![ServerAddress::parse(server).map_err(Error::InvalidClientOptions)?];
        }
        if source.is_some() {
            opts.credential.as_mut().unwrap().source = source;
        }
        Ok(())
    }

    fn split_uri(uri: &str) -> Result<(&str, &str)> {
        // find the index of the protocol separator
        // if it's not there, it's an error anyway
        let index = uri.find("://");
        if index.is_none() {
            return Err(Error::InvalidUriFormat(format!(
                "Invalid URI: {uri} does not contain `://` after protocol"
            )));
        }
        // split the uri into the protocol and the rest of the uri
        // safe to unwrap because we know the index is there
        let (protocol, rest) = uri.split_at(index.unwrap() + "://".len());
        Ok((protocol, rest))
    }

    fn contains_username_and_or_password(in_str: &str) -> bool {
        USERNAME_PASSWORD_REGEX.is_match(in_str)
    }

    /// construct_uri_for_parsing will construct a URI that can be parsed by the MongoDB driver if the
    /// the authMechanism is one of the following: SCRAM-SHA-1, SCRAM-SHA-256, or PLAIN. If the
    /// authMechanism is not one of these, the URI will be returned as is.
    fn construct_uri_for_parsing(&mut self, uri: &str) -> Result<String> {
        // if the uri already contains a username and (optionally) a password, we don't need to do anything
        if Self::contains_username_and_or_password(uri) {
            return Ok(uri.to_string());
        }
        if let Some(mechanism) = AUTH_MECHANISM_REGEX
            .captures(uri)
            .and_then(|cap| cap.name("mechanism").map(|s| s.as_str()))
        {
            match AuthMechanism::from_str(mechanism.to_uppercase().as_str()) {
                Ok(mechanism) => match mechanism {
                    AuthMechanism::ScramSha1
                    | AuthMechanism::ScramSha256
                    | AuthMechanism::Plain => self.inject_username_and_password_into_uri(uri),
                    _ => Ok(uri.to_string()),
                },
                Err(_) => Ok(uri.to_string()),
            }
        } else {
            // no authentication mechanism specified in the URI, so we'll treat it as an authless connection
            Ok(uri.to_string())
        }
    }

    fn inject_username_and_password_into_uri(&mut self, uri: &str) -> Result<String> {
        let (protocol, rest) = Self::split_uri(uri)?;
        let username = self.get_attribute(USER_KWS);
        let password = self.get_attribute(PWD_KWS);
        Ok(
            if let (Some(username), Some(password)) = (username, password) {
                format!("{protocol}{username}:{password}@{rest}")
            } else {
                uri.to_string()
            },
        )
    }

    async fn handle_uri(&mut self, uri: &str) -> Result<UserOptions> {
        let server = self.remove(SERVER_KWS);
        let source = AUTH_SOURCE_REGEX
            .captures(uri)
            .and_then(|cap| cap.name("source").map(|s| s.as_str()));

        // trust-dns-resolver has a performance issue on windows, so we'll use cloudflare's resolver
        // instead of the system resolver
        // https://github.com/mongodb/mongo-rust-driver?tab=readme-ov-file#windows-dns-note
        let uri = &self.construct_uri_for_parsing(uri)?;
        let parse_func = || async {
            if cfg!(target_os = "windows") {
                ClientOptions::parse(uri)
                    .resolver_config(ResolverConfig::cloudflare())
                    .await
            } else {
                ClientOptions::parse(uri).await
            }
        };
        let mut client_options = parse_func().await.map_err(Error::InvalidClientOptions)?;

        if client_options.credential.is_some() {
            // user name set as attribute should supercede mongo uri
            let user = self.remove(USER_KWS);
            if user.is_some() {
                client_options.credential.as_mut().unwrap().username = user;
            }
            // password set as attribute should supercede mongo uri
            let pwd = self.remove(PWD_KWS);
            if pwd.is_some() {
                client_options.credential.as_mut().unwrap().password = pwd;
            }
            Self::check_client_opts_credentials(&client_options)?;
        } else {
            // if the credentials were not set in the mongo uri, then user and pwd are _required_ to be
            // set as attributes.
            let user = self.remove_mandatory_attribute(USER_KWS)?;
            let pwd = self.remove_mandatory_attribute(PWD_KWS)?;
            client_options.credential =
                Some(Credential::builder().username(user).password(pwd).build());
        }

        Self::set_server_and_source(&mut client_options, server, source.map(String::from))?;

        let app_name = self.handle_app_name(client_options.app_name);
        let driver_name = self.handle_driver_info(app_name.as_ref().unwrap());
        client_options.app_name = app_name;
        client_options.driver_info = Some(driver_name);

        let uuid_representation = ConnectionString::parse(uri)
            .map_err(Error::InvalidClientOptions)?
            .uuid_representation;

        match client_options
            .credential
            .as_ref()
            .unwrap()
            .mechanism
            .as_ref()
        {
            Some(AuthMechanism::MongoDbX509) => {
                client_options.credential.as_mut().unwrap().username = None;
                client_options.credential.as_mut().unwrap().password = None;
            }
            Some(AuthMechanism::MongoDbOidc) => {
                use futures::future::FutureExt;
                let cred = client_options.credential.as_mut().unwrap();
                cred.oidc_callback = mongodb::options::oidc::Callback::human(move |c| {
                    async move { crate::oidc_auth::oidc_call_back(c).await }.boxed()
                });
                // Unset the password and username if they are empty strings.
                // This is to accommodate tools like Power BI that require adding empty username and password fields.
                // Note: OIDC (OpenID Connect) never uses a password.
                cred.password = None;
                cred.username = cred.username.as_ref().and_then(|x| {
                    if x.is_empty() {
                        None
                    } else {
                        Some(x.clone())
                    }
                });
            }
            _ => {}
        }

        Ok(UserOptions {
            client_options,
            uuid_representation,
        })
    }

    fn handle_no_uri(&mut self) -> Result<UserOptions> {
        let user = self.remove_mandatory_attribute(USER_KWS)?;
        let pwd = self.remove_mandatory_attribute(PWD_KWS)?;
        let server = self.remove_mandatory_attribute(SERVER_KWS)?;
        let cred = Credential::builder().username(user).password(pwd).build();
        let app_name = self.handle_app_name(None);
        let driver_name = self.handle_driver_info(app_name.as_ref().unwrap());
        Ok(UserOptions {
            client_options: ClientOptions::builder()
                .hosts(vec![
                    ServerAddress::parse(server).map_err(Error::InvalidClientOptions)?
                ])
                .credential(cred)
                .app_name(app_name)
                .driver_info(driver_name)
                .build(),
            uuid_representation: None,
        })
    }

    // handle_driver_info sets the driver name correctly for telemetry purposes.
    // In all instances, it will append "mongodb-odbc" to the driver name. If
    // the APPNAME ODBC URI property is provided and contains "powerbi-connector",
    // it will also append "powerbi-connector"
    // APPNAME.contains("powerbi-connector") -> "mongodb-odbc|powerbi-connector"
    // !APPNAME.contains("powerbi-connector") -> "mongodb-odbc"
    // this will materialize in telemetry as "mongo-rust-driver|mongodb-odbc|powerbi-connector"
    fn handle_driver_info(&self, app_name: &'_ str) -> DriverInfo {
        let driver_info = DriverInfo::builder();
        let driver_name = if app_name.contains(POWERBI_CONNECTOR) {
            format!("{DRIVER_SHORT_NAME}|{POWERBI_CONNECTOR}")
        } else {
            DRIVER_SHORT_NAME.to_string()
        };
        driver_info.name(driver_name).build()
    }

    // handle_app_name sets the appname correctly for telemetry purposes.
    // In all instances, it will set the base appname as "mongo-odbc+<version>"
    // If the APPNAME ODBC URI parameter is set and contains "powerbi-connector",
    // it will include the APPNAME supplied, i.e. "mongodb-odbc+<version>|powerbi_connector+<version>"
    // If an appName is included in the MongoDB URI, it will append that, i.e.
    // "mongodb-odbc+<version>|powerbi_connector+<version>|<appName>"
    // or "mongodb-odbc+<version>|<appName>" if no ODBC URI APPNAME is specified but there
    // is one specified in the MongoDB URI.
    fn handle_app_name(&mut self, app_name: Option<String>) -> Option<String> {
        if let Some(ref app_name) = app_name {
            log::info!("Connecting with : {app_name}");
        }
        Some(
            vec![
                Some(DEFAULT_APP_NAME.to_string()),
                self.remove(&[APPNAME]),
                app_name,
            ]
            .into_iter()
            .flatten()
            .fold(String::new(), |acc, x| {
                if acc.is_empty() {
                    x
                } else {
                    format!("{acc}|{x}")
                }
            }),
        )
    }
}

mod unit {

    mod test_username_password_detection {
        #[test]
        fn test_username_password_detection() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::contains_username_and_or_password(
                "foo:bar@localhost"
            ));
            assert!(ODBCUri::contains_username_and_or_password("foo@localhost"));
            assert!(!ODBCUri::contains_username_and_or_password(
                "localhost:27017"
            ));
            assert!(!ODBCUri::contains_username_and_or_password(
                "mongodb://localhost:27017/abc?authMechanism=SCRAM-SHA-1"
            ));
        }
    }

    mod test_uri_construction {
        #[test]
        fn test_no_auth_mechanism_specified_is_unmodified() {
            use crate::odbc_uri::ODBCUri;
            let uri = "mongodb://localhost:27017";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), uri);
        }

        #[test]
        fn test_scram_sha1_specified() {
            use crate::odbc_uri::ODBCUri;
            let uri = "mongodb://localhost:27017/abc?authMechanism=SCRAM-SHA-1";
            let expected = "mongodb://foo:bar@localhost:27017/abc?authMechanism=SCRAM-SHA-1";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), expected);
        }

        #[test]
        fn test_scram_sha_256_specified() {
            use crate::odbc_uri::ODBCUri;
            let uri = "mongodb://localhost:27017/abc?authMechanism=SCRAM-SHA-256";
            let expected = "mongodb://foo:bar@localhost:27017/abc?authMechanism=SCRAM-SHA-256";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar;")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), expected);
        }

        #[test]
        fn test_plain_specified() {
            use crate::odbc_uri::ODBCUri;
            let uri = "mongodb://localhost:27017/abc?authSource=$external&authMechanism=PLAIN";
            let expected =
                "mongodb://foo:bar@localhost:27017/abc?authSource=$external&authMechanism=PLAIN";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), expected);
        }

        #[test]
        fn test_mechanism_not_recognized() {
            use crate::odbc_uri::ODBCUri;
            let uri = "mongodb://localhost:27017/abc?authMechanism=SCRAM-SHA-512";
            let expected = "mongodb://localhost:27017/abc?authMechanism=SCRAM-SHA-512";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), expected);
        }

        #[test]
        fn test_x509_does_not_modify_uri() {
            use crate::odbc_uri::ODBCUri;
            let uri =
                "mongodb://localhost:27017/abc?authSource=$external&authMechanism=MONGODB-X509";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), uri);
        }

        #[test]
        fn test_aws_does_not_modify_uri() {
            use crate::odbc_uri::ODBCUri;
            let uri =
                "mongodb://localhost:27017/abc?authSource=$external&authMechanism=MONGODB-AWS";
            let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
            assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), uri);
        }
    }

    mod get_next_attribute {
        #[test]
        fn get_unbraced() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "foo".to_string(), None),
                ODBCUri::get_next_attribute("DRIVER=foo".to_string())
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_braced() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "fo[]=o".to_string(), None),
                ODBCUri::get_next_attribute("DRIVER={fo[]=o}".to_string())
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_unbraced_with_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                (
                    "driver".to_string(),
                    "foo".to_string(),
                    Some("UID=stuff".to_string())
                ),
                ODBCUri::get_next_attribute("DRIVER=foo;UID=stuff".to_string())
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_braced_with_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                (
                    "driver".to_string(),
                    "fo[]=o".to_string(),
                    Some("UID=stuff".to_string())
                ),
                ODBCUri::get_next_attribute("DRIVER={fo[]=o};UID=stuff".to_string())
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_with_non_keyword_in_keyword_position_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: 'stuff' is not a valid URI keyword",
                format!(
                    "{}",
                    ODBCUri::get_next_attribute("stuff=stuff;".to_string()).unwrap_err()
                )
            );
        }
    }

    mod handle_braced_value {
        #[test]
        fn no_closing_brace_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: attribute value beginning with '{' must end with '}'",
                format!(
                    "{}",
                    ODBCUri::handle_braced_value("stuff;stuff").unwrap_err()
                )
            );
        }

        #[test]
        fn ends_with_brace() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), None),
                ODBCUri::handle_braced_value("stuff}").unwrap()
            );
        }

        #[test]
        fn ends_with_semi() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), None),
                ODBCUri::handle_braced_value("stuff};").unwrap()
            );
        }

        #[test]
        fn has_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), Some("DRIVER=foo".to_string())),
                ODBCUri::handle_braced_value("stuff};DRIVER=foo").unwrap()
            );
        }

        #[test]
        fn ends_with_brace_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff".to_string(), None),
                ODBCUri::handle_braced_value("stu%=[]}ff}").unwrap()
            );
        }

        #[test]
        fn ends_with_semi_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff".to_string(), None),
                ODBCUri::handle_braced_value("stu%=[]}ff};").unwrap()
            );
        }

        #[test]
        fn has_rest_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff".to_string(), Some("DRIVER=foo".to_string())),
                ODBCUri::handle_braced_value("stu%=[]}ff};DRIVER=foo").unwrap()
            );
        }
    }

    mod handle_unbraced_value {
        #[test]
        fn ends_with_empty() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), None),
                ODBCUri::handle_unbraced_value("stuff").unwrap()
            );
        }

        #[test]
        fn ends_with_semi() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), None),
                ODBCUri::handle_unbraced_value("stuff;").unwrap()
            );
        }

        #[test]
        fn has_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff".to_string(), Some("DRIVER=foo".to_string())),
                ODBCUri::handle_unbraced_value("stuff;DRIVER=foo").unwrap()
            );
        }
    }

    mod new {
        #[test]
        fn empty_uri_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("".to_string()).is_err());
        }

        #[test]
        fn string_foo_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("Foo".to_string()).is_err());
        }

        #[test]
        fn missing_equals_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("driver=Foo;Bar".to_string()).is_err());
        }

        #[test]
        fn one_attribute_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(map! {"driver".to_string() => "Foo".to_string()});
            assert_eq!(expected, ODBCUri::new("Driver=Foo".to_string()).unwrap());
        }

        #[test]
        fn two_attributes_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "server".to_string() => "bAr".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;SERVER=bAr".to_string()).unwrap()
            );
        }

        #[test]
        fn standard_types_test() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "simple_types_only".to_string() => "0".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;simple_types_only=0".to_string()).unwrap()
            );
        }

        #[test]
        fn enable_max_string_length_test() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "enable_max_string_length".to_string() => "1".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;enable_max_string_length=1".to_string()).unwrap()
            );
        }

        #[test]
        fn repeated_attribute_selects_first() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "server".to_string() => "bAr".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;SERVER=bAr;Driver=F".to_string()).unwrap()
            );
        }

        #[test]
        fn two_attributes_with_trailing_semi_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "server".to_string() => "bAr".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;SERVER=bAr;".to_string()).unwrap()
            );
        }

        #[test]
        fn two_attributes_with_triple_trailing_semis_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "Foo".to_string(), "server".to_string() => "bAr".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;;;SERVER=bAr;;;".to_string()).unwrap()
            );
        }

        #[test]
        fn log_level() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(
                map! {"driver".to_string() => "foo".to_string(), "server".to_string() => "bAr".to_string(), "loglevel".to_string() => "debug".to_string()},
            );
            assert_eq!(
                expected,
                ODBCUri::new("Driver=foo;SERVER=bAr;LOGLEVEL=debug".to_string()).unwrap()
            );
        }
    }

    #[cfg(test)]
    mod try_into_client_options {
        use mongodb::options::ClientOptions;

        #[tokio::test(flavor = "current_thread")]
        async fn missing_server_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: server is required for a valid Mongo ODBC Uri",
                format!(
                    "{}",
                    ODBCUri::new("USER=foo;PWD=bar".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .await
                        .unwrap_err()
                )
            );
        }
        #[tokio::test(flavor = "current_thread")]
        async fn missing_pwd_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
            "Invalid Uri: One of [\"password\", \"pwd\"] is required for a valid Mongo ODBC Uri",
            format!(
                "{}",
                ODBCUri::new("USER=foo;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options().await
                    .unwrap_err()
            )
        );
        }
        #[tokio::test(flavor = "current_thread")]
        async fn missing_user_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: One of [\"uid\", \"user\"] is required for a valid Mongo ODBC Uri",
                format!(
                    "{}",
                    ODBCUri::new("PWD=bar;SERVER=127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .await
                        .unwrap_err()
                )
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn use_pwd_server_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .await
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .await
                    .unwrap()
                    .client_options
                    .credential
                    .unwrap()
                    .password
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uid_instead_of_user_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .await
                    .unwrap()
                    .credential
                    .unwrap()
                    .username,
                ODBCUri::new("UID=foo;PWD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .await
                    .unwrap()
                    .client_options
                    .credential
                    .unwrap()
                    .username
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn password_instead_of_pwd_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .await
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("UID=foo;PassworD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .await
                    .unwrap()
                    .client_options
                    .credential
                    .unwrap()
                    .password
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uri_with_embedded_user_and_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                .await
                .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("URI=mongodb://foo:bar@127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap()
                .client_options;
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uri_seperate_user_and_password_replace_embedded() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo2:bar2@127.0.0.1:27017")
                .await
                .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new(
                "USER=foo2;PWD=bar2;URI=mongodb://foo:bar@127.0.0.1:27017".to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap()
            .client_options;
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uri_with_separate_user_and_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                .await
                .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("USER=foo;PWD=bar;URI=mongodb://127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap()
                .client_options;
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uri_with_separate_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                .await
                .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("PWD=bar;URI=mongodb://foo@127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap()
                .client_options;
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn credless_uri_without_user_and_password_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Err(InvalidUriFormat(\"One of [\\\"uid\\\", \\\"user\\\"] is required for a valid Mongo ODBC Uri\"))".to_string(),
                format!(
                    "{:?}",
                    ODBCUri::new("URI=mongodb://127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .await
                )
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn credless_uri_with_user_and_no_password_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Err(InvalidUriFormat(\"One of [\\\"password\\\", \\\"pwd\\\"] is required for a valid Mongo ODBC Uri\"))".to_string(),
                format!(
                    "{:?}",
                    ODBCUri::new("URI=mongodb://foo@127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .await
                )
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ldap_correctness() {
            use crate::odbc_uri::ODBCUri;
            let odbc_str = "URI=mongodb://localhost/abc?authSource=$external&authMechanism=PLAIN;UID=foo;PWD=bar";
            let actual = ODBCUri::new(odbc_str.to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap()
                .client_options
                .credential
                .unwrap();
            assert_eq!(
                Some(mongodb::options::AuthMechanism::Plain),
                actual.mechanism
            );
            assert_eq!(Some("$external".to_string()), actual.source);
            assert_eq!(Some("foo".to_string()), actual.username);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn auth_source_correctness() {
            use crate::odbc_uri::ODBCUri;
            for (expected, uri) in [
                (Some("authDB".to_string()), "URI=mongodb://localhost/?authSource=authDB;UID=foo;PWD=bar"),
                (None, "URI=mongodb://localhost/;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?auTHSource=aut:hD@B;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?auTHSource=aut:hD@B&appName=tgg#fed;UID=foo;PWD=bar"),
                (Some("$external".to_string()), "URI=mongodb://uid:pwd@localhost/?authSource=$external&appName=tgg#fed;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?appName=test&auTHSource=aut:hD@B;UID=foo;PWD=bar"),
                (Some("jfhbgvhj".to_string()), "URI=mongodb://localhost/?ssl=true&appName='myauthSource=aut:hD@B'&authSource=jfhbgvhj;UID=f;PWD=b" ),
            ] {
                let actual = ODBCUri::new(uri.to_string()).unwrap().try_into_client_options().await.unwrap().client_options.credential.unwrap().source;
            assert_eq!(
                expected, actual, "expected {expected:?}, got {actual:?}" );
            }
        }

        #[tokio::test(flavor = "current_thread")]
        async fn uri_seperate_server_replaces_embedded() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo2:bar2@127.0.0.2:27017")
                .await
                .unwrap();
            let opts = ODBCUri::new(
                "SERVER=127.0.0.2:27017;URI=mongodb://foo:bar@127.0.0.1:27017".to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap()
            .client_options;
            assert_eq!(expected_opts.hosts[0], opts.hosts[0]);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn supplied_args_override_dsn_args() {
            // we leave dsn out of the uri so that it doesn't try to query the registry in the test
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse(
                "mongodb://foo:bar@www.atlas.net:27017/?authSource=authDB&ssl=true",
            )
            .await
            .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("UID=foo;PWD=bar;SERVER=www.atlas.net:27017;User=foo2;Password=bar2;uri=mongodb://localhost:29000/?authSource=authDB&ssl=true".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap().client_options;
            let cred = opts.credential.unwrap();
            assert_eq!(expected_opts.hosts[0], opts.hosts[0]);
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
        }

        #[tokio::test(flavor = "current_thread")]
        async fn no_app_name_in_odbc_uri_or_mongodb_uri_still_shows_odbc_driver_version() {
            use crate::odbc_uri::ODBCUri;
            use constants::DEFAULT_APP_NAME;
            let uri_opts =
                ODBCUri::new("USER=foo;PWD=bar;URI=mongodb://localhost:27017/".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .await
                    .unwrap();
            assert_eq!(*DEFAULT_APP_NAME, uri_opts.client_options.app_name.unwrap());
        }

        #[tokio::test(flavor = "current_thread")]
        async fn app_name_in_odbc_uri_and_no_mongodb_uri() {
            use crate::odbc_uri::ODBCUri;
            use constants::DEFAULT_APP_NAME;
            let uri_opts = ODBCUri::new(
                "USER=foo;PWD=bar;SERVER=localhost:27017;APPNAME=powerbi-connector+1.0.0"
                    .to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap();
            assert_eq!(
                format!("{}|powerbi-connector+1.0.0", *DEFAULT_APP_NAME),
                uri_opts.client_options.app_name.unwrap()
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn no_app_name_in_odbc_uri_and_no_mongodb_uri() {
            use crate::odbc_uri::ODBCUri;
            use constants::DEFAULT_APP_NAME;
            let uri_opts = ODBCUri::new("USER=foo;PWD=bar;SERVER=localhost:27017;".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap();
            assert_eq!(*DEFAULT_APP_NAME, uri_opts.client_options.app_name.unwrap());
        }

        #[tokio::test(flavor = "current_thread")]
        async fn app_name_in_odbc_uri_shows_with_odbc_driver_version_added() {
            use crate::odbc_uri::ODBCUri;
            use constants::DEFAULT_APP_NAME;
            let uri_opts = ODBCUri::new(
                "USER=foo;PWD=bar;URI=mongodb://localhost:27017/;APPNAME=powerbi-connector+1.0.0"
                    .to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap();

            assert_eq!(
                format!("{}|powerbi-connector+1.0.0", *DEFAULT_APP_NAME),
                uri_opts.client_options.app_name.unwrap()
            );
        }
        #[tokio::test(flavor = "current_thread")]
        async fn app_name_in_odbc_uri_and_mongodb_uri_chains_and_adds_odbc_driver_version() {
            use crate::odbc_uri::ODBCUri;
            use constants::DEFAULT_APP_NAME;
            let uri_opts = ODBCUri::new(
                "USER=foo;PWD=bar;URI=mongodb://localhost:27017/?appName=foo;APPNAME=powerbi-connector+1.0.0"
                    .to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap();

            assert_eq!(
                format!("{}|powerbi-connector+1.0.0|foo", *DEFAULT_APP_NAME),
                uri_opts.client_options.app_name.unwrap()
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn driver_info_with_powerbi_provided() {
            use crate::odbc_uri::ODBCUri;
            use constants::DRIVER_SHORT_NAME;
            let uri_opts = ODBCUri::new(
                "USER=foo;PWD=bar;URI=mongodb://localhost:27017/?appName=foo;APPNAME=powerbi-connector+1.0.0"
                    .to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap();

            assert_eq!(
                format!("{DRIVER_SHORT_NAME}|powerbi-connector",),
                uri_opts.client_options.driver_info.unwrap().name
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn driver_info_with_no_powerbi_provided() {
            use crate::odbc_uri::ODBCUri;
            use constants::DRIVER_SHORT_NAME;
            let uri_opts = ODBCUri::new(
                "USER=foo;PWD=bar;URI=mongodb://localhost:27017/?appName=foo;".to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .await
            .unwrap();

            assert_eq!(
                DRIVER_SHORT_NAME,
                uri_opts.client_options.driver_info.unwrap().name
            );
        }

        #[tokio::test(flavor = "current_thread")]
        async fn driver_info_with_no_mongodb_uri_and_no_odbc_uri_appname() {
            use crate::odbc_uri::ODBCUri;
            use constants::DRIVER_SHORT_NAME;
            let uri_opts = ODBCUri::new("USER=foo;PWD=bar;SERVER=localhost:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .await
                .unwrap();

            assert_eq!(
                DRIVER_SHORT_NAME,
                uri_opts.client_options.driver_info.unwrap().name
            );
        }
    }
}
