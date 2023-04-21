use crate::err::{Error, Result};
use constants::{DEFAULT_APP_NAME, DRIVER_METRICS_VERSION};
use lazy_static::lazy_static;
use mongodb::options::{ClientOptions, Credential, ServerAddress};
use regex::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use shared_sql_utils::Dsn;
use std::collections::HashMap;

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

const POWERBI_CONNECTOR: &str = "powerbi-connector";

const URI_KWS: &[&str] = &[URI];
const USER_KWS: &[&str] = &[UID, USER];
const PWD_KWS: &[&str] = &[PASSWORD, PWD];
const SERVER_KWS: &[&str] = &[SERVER];

lazy_static! {
    static ref KEYWORDS: RegexSet = RegexSetBuilder::new(
        [DATABASE, DRIVER, DSN, PASSWORD, PWD, SERVER, USER, UID, URI, APPNAME, LOGLEVEL]
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
        for (i, c) in input.chars().enumerate() {
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

    // remove will remove the first value with a given one of the names passed, assuming all names
    // are synonyms.
    pub fn remove(&mut self, names: &[&str]) -> Option<String> {
        for name in names.iter() {
            let ret = self.0.remove(&name.to_string());
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
    pub fn try_into_client_options(&mut self) -> Result<ClientOptions> {
        let uri = self.remove(URI_KWS);
        if let Some(uri) = uri {
            return self.handle_uri(&uri);
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
        mut opts: &mut ClientOptions,
        server: Option<String>,
        source: Option<String>,
    ) -> Result<()> {
        // server should supercede that specified in the uri, if specified.
        if let Some(server) = server {
            opts.hosts = vec![ServerAddress::parse(server).map_err(Error::InvalidClientOptions)?];
        }
        if source.is_some() {
            opts.credential.as_mut().unwrap().source = source.map(String::from);
        }
        Ok(())
    }

    fn handle_uri(&mut self, uri: &str) -> Result<ClientOptions> {
        let server = self.remove(SERVER_KWS);
        let source = AUTH_SOURCE_REGEX
            .captures(uri)
            .and_then(|cap| cap.name("source").map(|s| s.as_str()));
        let mut client_options = ClientOptions::parse(uri).map_err(Error::InvalidClientOptions)?;
        if client_options.credential.is_some() {
            // user name set as attribute should supercede mongo uri
            let user = self.remove(USER_KWS);
            if user.is_some() {
                client_options.credential.as_mut().unwrap().username = user.map(String::from);
            }
            // password set as attribute should supercede mongo uri
            let pwd = self.remove(PWD_KWS);
            if pwd.is_some() {
                client_options.credential.as_mut().unwrap().password = pwd.map(String::from);
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
        client_options.app_name = client_options.app_name.or(self.handle_app_name());
        Ok(client_options)
    }

    fn handle_no_uri(&mut self) -> Result<ClientOptions> {
        let user = self.remove_mandatory_attribute(USER_KWS)?;
        let pwd = self.remove_mandatory_attribute(PWD_KWS)?;
        let server = self.remove_mandatory_attribute(SERVER_KWS)?;
        let cred = Credential::builder().username(user).password(pwd).build();
        Ok(ClientOptions::builder()
            .hosts(vec![
                ServerAddress::parse(server).map_err(Error::InvalidClientOptions)?
            ])
            .credential(cred)
            .app_name(self.handle_app_name())
            .build())
    }

    fn handle_app_name(&mut self) -> Option<String> {
        self.remove(&[APPNAME])
            .map(|s| {
                if s == POWERBI_CONNECTOR {
                    format!("{}+{}", POWERBI_CONNECTOR, DRIVER_METRICS_VERSION.as_str())
                } else {
                    s
                }
            })
            .or(Some(DEFAULT_APP_NAME.to_string()))
    }
}

mod unit {
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

        use crate::odbc_uri::POWERBI_CONNECTOR;

        #[test]
        fn missing_server_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: server is required for a valid Mongo ODBC Uri",
                format!(
                    "{}",
                    ODBCUri::new("USER=foo;PWD=bar".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .unwrap_err()
                )
            );
        }
        #[test]
        fn missing_pwd_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
            "Invalid Uri: One of [\"password\", \"pwd\"] is required for a valid Mongo ODBC Uri",
            format!(
                "{}",
                ODBCUri::new("USER=foo;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .unwrap_err()
            )
        );
        }
        #[test]
        fn missing_user_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: One of [\"uid\", \"user\"] is required for a valid Mongo ODBC Uri",
                format!(
                    "{}",
                    ODBCUri::new("PWD=bar;SERVER=127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                        .unwrap_err()
                )
            );
        }

        #[test]
        fn use_pwd_server_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .credential
                    .unwrap()
                    .password
            );
        }

        #[test]
        fn uid_instead_of_user_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .unwrap()
                    .credential
                    .unwrap()
                    .username,
                ODBCUri::new("UID=foo;PWD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .credential
                    .unwrap()
                    .username
            );
        }

        #[test]
        fn password_instead_of_pwd_works() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017")
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("UID=foo;PassworD=bar;SERVER=127.0.0.1:27017".to_string())
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .credential
                    .unwrap()
                    .password
            );
        }

        #[test]
        fn uri_with_embedded_user_and_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017").unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("URI=mongodb://foo:bar@127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .unwrap();
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[test]
        fn uri_seperate_user_and_password_replace_embedded() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts =
                ClientOptions::parse("mongodb://foo2:bar2@127.0.0.1:27017").unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new(
                "USER=foo2;PWD=bar2;URI=mongodb://foo:bar@127.0.0.1:27017".to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .unwrap();
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[test]
        fn uri_with_separate_user_and_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017").unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("USER=foo;PWD=bar;URI=mongodb://127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .unwrap();
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[test]
        fn uri_with_separate_password_works() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017").unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("PWD=bar;URI=mongodb://foo@127.0.0.1:27017".to_string())
                .unwrap()
                .try_into_client_options()
                .unwrap();
            let cred = opts.credential.unwrap();
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
            assert_eq!(expected_opts.hosts, opts.hosts);
        }

        #[test]
        fn credless_uri_without_user_and_password_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Err(InvalidUriFormat(\"One of [\\\"uid\\\", \\\"user\\\"] is required for a valid Mongo ODBC Uri\"))".to_string(),
                format!(
                    "{:?}",
                    ODBCUri::new("URI=mongodb://127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                )
            );
        }

        #[test]
        fn credless_uri_with_user_and_no_password_is_error() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Err(InvalidUriFormat(\"One of [\\\"password\\\", \\\"pwd\\\"] is required for a valid Mongo ODBC Uri\"))".to_string(),
                format!(
                    "{:?}",
                    ODBCUri::new("URI=mongodb://foo@127.0.0.1:27017".to_string())
                        .unwrap()
                        .try_into_client_options()
                )
            );
        }

        #[test]
        fn auth_source_correctness() {
            use crate::odbc_uri::ODBCUri;
            for (source, uri) in [
                (Some("authDB".to_string()), "URI=mongodb://localhost/?authSource=authDB;UID=foo;PWD=bar"),
                (None, "URI=mongodb://localhost/;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?auTHSource=aut:hD@B;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?auTHSource=aut:hD@B&appName=tgg#fed;UID=foo;PWD=bar"),
                (Some("$external".to_string()), "URI=mongodb://uid:pwd@localhost/?authSource=$external&appName=tgg#fed;UID=foo;PWD=bar"),
                (Some("aut:hD@B".to_string()), "URI=mongodb://localhost/?appName=test&auTHSource=aut:hD@B;UID=foo;PWD=bar"),
                (Some("jfhbgvhj".to_string()), "URI=mongodb://localhost/?ssl=true&appName='myauthSource=aut:hD@B'&authSource=jfhbgvhj;UID=f;PWD=b" ),
            ] {
            assert_eq!(
                source, ODBCUri::new(uri.to_string()).unwrap().try_into_client_options().unwrap().credential.unwrap().source);
            }
        }

        #[test]
        fn app_name_correctness() {
            use crate::odbc_uri::{ODBCUri, DEFAULT_APP_NAME};
            use constants::DRIVER_METRICS_VERSION;
            for (source, uri) in [
                (Some("app".to_string()), "URI=mongodb://localhost/?authSource=authDB;UID=foo;PWD=bar;appname=app"),
                (Some(DEFAULT_APP_NAME.to_string()), "URI=mongodb://localhost/;UID=foo;PWD=bar"),
                (Some("powerbi".to_string()), "URI=mongodb://localhost/?aPpNaMe=powerbi;UID=foo;PWD=bar"),
                (Some("encabulator".to_string()), "URI=mongodb://localhost/?ssl=true&APPNAME=encabulator&authSource=jfhbgvhj;UID=f;PWD=b;APPNAME=powerbi-connector"),
                (Some(format!("{}+{}",POWERBI_CONNECTOR, DRIVER_METRICS_VERSION.as_str())), format!("URI=mongodb://localhost/?ssl=true&authSource=jfhbgvhj;UID=f;PWD=b;APPNAME={POWERBI_CONNECTOR}").as_str()),
            ] {
            assert_eq!(
                source, ODBCUri::new(uri.to_string()).unwrap().try_into_client_options().unwrap().app_name);
            }
        }

        #[test]
        fn uri_seperate_server_replaces_embedded() {
            use crate::odbc_uri::ODBCUri;
            let expected_opts =
                ClientOptions::parse("mongodb://foo2:bar2@127.0.0.2:27017").unwrap();
            let opts = ODBCUri::new(
                "SERVER=127.0.0.2:27017;URI=mongodb://foo:bar@127.0.0.1:27017".to_string(),
            )
            .unwrap()
            .try_into_client_options()
            .unwrap();
            assert_eq!(expected_opts.hosts[0], opts.hosts[0]);
        }

        #[test]
        fn supplied_args_override_dsn_args() {
            // we leave dsn out of the uri so that it doesn't try to query the registry in the test
            use crate::odbc_uri::ODBCUri;
            let expected_opts = ClientOptions::parse(
                "mongodb://foo:bar@www.atlas.net:27017/?authSource=authDB&ssl=true",
            )
            .unwrap();
            let expected_cred = expected_opts.credential.unwrap();
            let opts = ODBCUri::new("UID=foo;PWD=bar;SERVER=www.atlas.net:27017;User=foo2;Password=bar2;uri=mongodb://localhost:29000/?authSource=authDB&ssl=true".to_string())
                .unwrap()
                .try_into_client_options()
                .unwrap();
            let cred = opts.credential.unwrap();
            assert_eq!(expected_opts.hosts[0], opts.hosts[0]);
            assert_eq!(expected_cred.username, cred.username);
            assert_eq!(expected_cred.password, cred.password);
        }
    }
}
