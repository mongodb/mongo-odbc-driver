use crate::errors::{ODBCError, Result};
use std::collections::BTreeMap;

// TODO SQL-990: These errors will probably change.
const NOT_EMPTY_ERROR: &str = "uri must not be empty";
const EQUAL_ERROR: &str = "all uri atttributes must be of the form key=value";

// TODO SQL-990: Audit these mandatory attributes
const USER: &[&str] = &["user", "uid"];
const PWD: &[&str] = &["pwd", "password"];
const SERVER: &[&str] = &["server"];
const SSL: &[&str] = &["ssl"];

#[derive(Debug, PartialEq, Eq)]
pub struct ODBCUri<'a>(BTreeMap<String, &'a str>);

impl<'a> ODBCUri<'a> {
    pub(crate) fn new(odbc_uri: &'a str) -> Result<ODBCUri<'a>> {
        if odbc_uri.is_empty() {
            return Err(ODBCError::InvalidUriFormat(NOT_EMPTY_ERROR.to_string()));
        }
        // TODO SQL-990: Support the actual ODBC spec with regards to special characters in attributes
        // the algorithm will most likely need to be a state machine over each character in the
        // odbc_uri string.
        odbc_uri
            .split(';')
            .filter(|attr| !attr.is_empty())
            .map(|attr| {
                // now split each attribute pair on '='
                let mut sp = attr.split('=').collect::<Vec<_>>();
                if sp.len() != 2 {
                    return Err(ODBCError::InvalidUriFormat(EQUAL_ERROR.to_string()));
                }
                // ODBC attribute keys are case insensitive, so we lowercase the keys
                Ok((sp.remove(0).to_lowercase(), sp.remove(0)))
            })
            .collect::<Result<BTreeMap<_, _>>>()
            .map(ODBCUri)
    }

    // remove will remove the first value with a given one of the names passed, assuming all names
    // are synonyms.
    pub fn remove(&mut self, names: &[&str]) -> Option<&'a str> {
        for name in names.iter() {
            let ret = self.0.remove(&name.to_string());
            if ret.is_some() {
                return ret;
            }
        }
        None
    }

    // remove_or_else is the same as remove but with a default thunk.
    pub fn remove_or_else(&mut self, f: impl Fn() -> &'a str, names: &[&str]) -> &'a str {
        self.remove(names).unwrap_or_else(f)
    }

    // remove_mandatory_attribute will find an attribute that must exist and transfer ownership to
    // the caller. It accepts a slice of names that will be checked in order for names that are
    // synonyms (e.g., uid and user are both viable attribute names for a user). If both names
    // exist it will only find the first.
    fn remove_mandatory_attribute(&mut self, names: &[&str]) -> Result<&'a str> {
        // len is used only for more informative error messages.
        let mut len = 0;
        for (i, name) in names.iter().enumerate() {
            len = i;
            let result = self.0.remove(&name.to_string());
            if let Some(ret) = result {
                return Ok(ret);
            }
        }
        if len == 1 {
            Err(ODBCError::InvalidUriFormat(format!(
                "{} is required for a valid Mongo ODBC Uri",
                names[0]
            )))
        } else {
            Err(ODBCError::InvalidUriFormat(format!(
                "One of {:?} is required for a valid Mongo ODBC Uri",
                names
            )))
        }
    }

    // remove_mongo_uri converts this ODBCUri to a mongo_uri String. It will
    // remove all the attributes necessary to make a mongo_uri. This is destructive!
    pub fn remove_mongo_uri(&mut self) -> Result<String> {
        let user = self.remove_mandatory_attribute(USER)?;
        let pwd = self.remove_mandatory_attribute(PWD)?;
        // TODO SQL-990: Support the PORT attribute, right now the only way to specify PORT is as
        // part of SERVER. If ports are specified in both SERVER and PORT and they do not match it
        // should be an error (I think, check the spec if it says...).
        let server = self.remove_mandatory_attribute(SERVER)?;
        let ssl = self.remove(SSL);
        // TODO SQL-990: we may wish to support more attributes as options.
        // If we do, add more tests to cover them.
        let ssl_string =
            if ssl.is_some() && ssl.unwrap() != "0" && ssl.unwrap().to_lowercase() != "false" {
                "?ssl=true"
            } else {
                ""
            };
        Ok(format!(
            "mongodb://{}:{}@{}{}",
            user, pwd, server, ssl_string
        ))
    }
}

mod unit {
    // TODO SQL-990: Add more tests to cover the ODBC spec with regards to special characters.
    #[test]
    fn test_new() {
        use super::*;
        use crate::map;

        assert!(ODBCUri::new("").is_err());
        assert!(ODBCUri::new("Foo").is_err());
        assert!(ODBCUri::new("driver=Foo;Bar").is_err());

        let expected = ODBCUri(map! {"driver".to_string() => "Foo"});
        assert_eq!(expected, ODBCUri::new("Driver=Foo").unwrap());

        let expected = ODBCUri(map! {"driver".to_string() => "Foo", "server".to_string() => "bAr"});
        assert_eq!(expected, ODBCUri::new("Driver=Foo;SERVER=bAr").unwrap());
        assert_eq!(expected, ODBCUri::new("Driver=Foo;SERVER=bAr;").unwrap());
    }

    #[test]
    fn test_remove_mongo_uri() {
        use super::*;
        assert!(ODBCUri::new("USER=foo;PWD=bar")
            .unwrap()
            .remove_mongo_uri()
            .is_err());
        assert!(ODBCUri::new("USER=foo;SERVER=127.0.0.1:27017")
            .unwrap()
            .remove_mongo_uri()
            .is_err());
        assert!(ODBCUri::new("PWD=bar;SERVER=127.0.0.1:27017")
            .unwrap()
            .remove_mongo_uri()
            .is_err());
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017".to_string(),
            ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017".to_string(),
            ODBCUri::new("UID=foo;PWD=bar;SERVER=127.0.0.1:27017")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017".to_string(),
            ODBCUri::new("UID=foo;PassworD=bar;SERVER=127.0.0.1:27017")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017".to_string(),
            ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=faLse")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017".to_string(),
            ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=0")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017?ssl=true".to_string(),
            ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=1")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
        assert_eq!(
            "mongodb://foo:bar@127.0.0.1:27017?ssl=true".to_string(),
            ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=true")
                .unwrap()
                .remove_mongo_uri()
                .unwrap()
        );
    }
}
