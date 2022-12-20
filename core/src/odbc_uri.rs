use crate::err::{Error, Result};
use lazy_static::lazy_static;
use mongodb::options::{ClientOptions, Credential, ServerAddress, Tls, TlsOptions};
use regex::{RegexSet, RegexSetBuilder};
use std::collections::{HashMap, HashSet};

const EMPTY_URI_ERROR: &str = "URI must not be empty";
const INVALID_ATTR_FORMAT_ERROR: &str = "all URI attributes must be of the form keyword=value";
const MISSING_CLOSING_BRACE_ERROR: &str = "attribute value beginning with '{' must end with '}'";

const URI: &[&str] = &["uri"];
const USER: &[&str] = &["uid", "user"];
const PWD: &[&str] = &["password", "pwd"];
const SERVER: &[&str] = &["server"];
const SSL: &[&str] = &["ssl", "tls"];

lazy_static! {
    static ref KEYWORDS: RegexSet = RegexSetBuilder::new([
        "^AUTH_SRC$",
        "^DATABASE$",
        "^DRIVER$",
        "^DSN$",
        "^PASSWORD$",
        "^PWD$",
        "^SERVER$",
        "^SSL$",
        "^TLS$",
        "^USER$",
        "^UID$",
        "^URI$",
    ])
    .case_insensitive(true)
    .build()
    .unwrap();

    // taken from the rust driver
    static ref ILLEGAL_DATABASE_CHARACTERS: HashSet<&'static char> = {
        ['/', '\\', ' ', '"', '$', '.'].iter().collect()
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct ODBCUri<'a>(HashMap<String, &'a str>);

impl<'a> ODBCUri<'a> {
    pub fn new(odbc_uri: &'a str) -> Result<ODBCUri<'a>> {
        if odbc_uri.is_empty() {
            return Err(Error::InvalidUriFormat(EMPTY_URI_ERROR.to_string()));
        }
        let mut input = odbc_uri;
        let mut ret = ODBCUri(HashMap::new());
        while let Some((keyword, value, rest)) = ODBCUri::get_next_attribute(input)? {
            // if attributes are repeated, the first is the one that is kept.
            ret.0.entry(keyword).or_insert(value);
            if rest.is_none() {
                return Ok(ret);
            }
            input = rest.unwrap();
        }
        Ok(ret)
    }

    fn get_next_attribute(odbc_uri: &'a str) -> Result<Option<(String, &'a str, Option<&'a str>)>> {
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
                "'{}' is not a valid URI keyword",
                keyword
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

    fn handle_braced_value(input: &'a str) -> Result<(&'a str, Option<&'a str>)> {
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
                return Ok((input.get(0..i - 1).unwrap(), rest));
            }
            if c == '}' {
                if i + 1 == input.len() {
                    return Ok((input.get(0..i).unwrap(), None));
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

    fn handle_unbraced_value(input: &'a str) -> Result<(&'a str, Option<&'a str>)> {
        let index = input.find(';');
        if index.is_none() {
            return Ok((input, None));
        }
        let (value, rest) = input.split_at(index.unwrap());
        if rest.len() == 1 {
            return Ok((value, None));
        }
        Ok((value, rest.get(1..)))
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
    // exist, it will only find the first.
    fn remove_mandatory_attribute(&mut self, names: &[&str]) -> Result<&'a str> {
        self.remove(names).ok_or_else(|| {
            if names.len() == 1 {
                Error::InvalidUriFormat(format!(
                    "{} is required for a valid Mongo ODBC Uri",
                    names[0]
                ))
            } else {
                Error::InvalidUriFormat(format!(
                    "One of {:?} is required for a valid Mongo ODBC Uri",
                    names
                ))
            }
        })
    }

    // try_into_client_options converts this ODBCUri to a mongo_uri String. It will
    // remove all the attributes necessary to make a mongo_uri. This is destructive!
    pub fn try_into_client_options(&mut self) -> Result<ClientOptions> {
        let uri = self.remove(URI);
        if uri.is_some() {
            return self.handle_uri(uri.unwrap());
        }
        self.handle_no_uri()
    }

    fn handle_uri(&mut self, uri: &str) -> Result<ClientOptions> {
        let mut client_options = ClientOptions::parse(uri)?;
        if client_options.credential.is_some() {
            let user = self.remove(USER);
            if user.is_some() {
                client_options.credential.as_mut().unwrap().username = user.map(String::from);
            }
            let pwd = self.remove(PWD);
            if pwd.is_some() {
                client_options.credential.as_mut().unwrap().password = pwd.map(String::from);
            }
            return Ok(client_options);
        }
        let user = self.remove(USER);
        let pwd = self.remove(PWD);
        client_options.credential = Some(
            Credential::builder()
                .username(user.map(String::from))
                .password(pwd.map(String::from))
                .build(),
        );
        Ok(client_options)
    }

    fn handle_no_uri(&mut self) -> Result<ClientOptions> {
        let user = self.remove_mandatory_attribute(USER)?;
        let pwd = self.remove_mandatory_attribute(PWD)?;
        let server = self.remove_mandatory_attribute(SERVER)?;
        let ssl = self.remove(SSL);
        let cred = Credential::builder()
            .username(user.to_string())
            .password(pwd.to_string())
            .build();
        let mut client_options = ClientOptions::builder()
            .hosts(vec![ServerAddress::parse(server)?])
            .credential(cred)
            .build();
        if ssl.is_some() && ssl.unwrap() != "0" && ssl.unwrap().to_lowercase() != "false" {
            client_options.tls = Some(Tls::Enabled(TlsOptions::builder().build()));
        };
        Ok(client_options)
    }
}

mod unit {
    mod get_next_attribute {
        #[test]
        fn get_unbraced() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "foo", None),
                ODBCUri::get_next_attribute("DRIVER=foo").unwrap().unwrap(),
            );
        }

        #[test]
        fn get_braced() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "fo[]=o", None),
                ODBCUri::get_next_attribute("DRIVER={fo[]=o}")
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_unbraced_with_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "foo", Some("UID=stuff")),
                ODBCUri::get_next_attribute("DRIVER=foo;UID=stuff")
                    .unwrap()
                    .unwrap(),
            );
        }

        #[test]
        fn get_braced_with_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("driver".to_string(), "fo[]=o", Some("UID=stuff")),
                ODBCUri::get_next_attribute("DRIVER={fo[]=o};UID=stuff")
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
                    ODBCUri::get_next_attribute("stuff=stuff;").unwrap_err()
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
                ("stuff", None),
                ODBCUri::handle_braced_value("stuff}").unwrap()
            );
        }

        #[test]
        fn ends_with_semi() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff", None),
                ODBCUri::handle_braced_value("stuff};").unwrap()
            );
        }

        #[test]
        fn has_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff", Some("DRIVER=foo")),
                ODBCUri::handle_braced_value("stuff};DRIVER=foo").unwrap()
            );
        }

        #[test]
        fn ends_with_brace_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff", None),
                ODBCUri::handle_braced_value("stu%=[]}ff}").unwrap()
            );
        }

        #[test]
        fn ends_with_semi_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff", None),
                ODBCUri::handle_braced_value("stu%=[]}ff};").unwrap()
            );
        }

        #[test]
        fn has_rest_special_chars() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stu%=[]}ff", Some("DRIVER=foo")),
                ODBCUri::handle_braced_value("stu%=[]}ff};DRIVER=foo").unwrap()
            );
        }
    }

    mod handle_unbraced_value {
        #[test]
        fn ends_with_empty() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff", None),
                ODBCUri::handle_unbraced_value("stuff").unwrap()
            );
        }

        #[test]
        fn ends_with_semi() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff", None),
                ODBCUri::handle_unbraced_value("stuff;").unwrap()
            );
        }

        #[test]
        fn has_rest() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ("stuff", Some("DRIVER=foo")),
                ODBCUri::handle_unbraced_value("stuff;DRIVER=foo").unwrap()
            );
        }
    }

    mod new {
        #[test]
        fn empty_uri_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("").is_err());
        }

        #[test]
        fn string_foo_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("Foo").is_err());
        }

        #[test]
        fn missing_equals_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert!(ODBCUri::new("driver=Foo;Bar").is_err());
        }

        #[test]
        fn one_attribute_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected = ODBCUri(map! {"driver".to_string() => "Foo"});
            assert_eq!(expected, ODBCUri::new("Driver=Foo").unwrap());
        }

        #[test]
        fn two_attributes_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected =
                ODBCUri(map! {"driver".to_string() => "Foo", "server".to_string() => "bAr"});
            assert_eq!(expected, ODBCUri::new("Driver=Foo;SERVER=bAr").unwrap());
        }

        #[test]
        fn repeated_attribute_selects_first() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected =
                ODBCUri(map! {"driver".to_string() => "Foo", "server".to_string() => "bAr"});
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;SERVER=bAr;Driver=F").unwrap()
            );
        }

        #[test]
        fn two_attributes_with_trailing_semi_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected =
                ODBCUri(map! {"driver".to_string() => "Foo", "server".to_string() => "bAr"});
            assert_eq!(expected, ODBCUri::new("Driver=Foo;SERVER=bAr;").unwrap());
        }

        #[test]
        fn two_attributes_with_triple_trailing_semis_works() {
            use crate::map;
            use crate::odbc_uri::ODBCUri;
            let expected =
                ODBCUri(map! {"driver".to_string() => "Foo", "server".to_string() => "bAr"});
            assert_eq!(
                expected,
                ODBCUri::new("Driver=Foo;;;SERVER=bAr;;;").unwrap()
            );
        }
    }

    #[cfg(test)]
    mod try_into_client_options {
        use mongodb::options::ClientOptions;

        #[test]
        fn missing_server_is_err() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                "Invalid Uri: server is required for a valid Mongo ODBC Uri",
                format!(
                    "{}",
                    ODBCUri::new("USER=foo;PWD=bar")
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
                ODBCUri::new("USER=foo;SERVER=127.0.0.1:27017")
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
                    ODBCUri::new("PWD=bar;SERVER=127.0.0.1:27017")
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
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017".to_string())
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017")
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
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017".to_string())
                    .unwrap()
                    .credential
                    .unwrap()
                    .username,
                ODBCUri::new("UID=foo;PWD=bar;SERVER=127.0.0.1:27017")
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
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017".to_string())
                    .unwrap()
                    .credential
                    .unwrap()
                    .password,
                ODBCUri::new("UID=foo;PassworD=bar;SERVER=127.0.0.1:27017")
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .credential
                    .unwrap()
                    .password
            );
        }
        // SSL=faLse should not set SSL option
        #[test]
        fn ssl_eq_false_should_not_set_ssl() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017".to_string())
                    .unwrap()
                    .tls,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=faLse")
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .tls
            );
        }

        #[test]
        fn ssl_eq_0_should_not_set_ssl() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017".to_string())
                    .unwrap()
                    .tls,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=0")
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .tls
            );
        }

        #[test]
        fn ssl_eq_1_should_set_ssl_to_true() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017/?ssl=true".to_string())
                    .unwrap()
                    .tls,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=1")
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .tls
            );
        }

        #[test]
        fn ssl_eq_true_should_set_ssl_to_true() {
            use crate::odbc_uri::ODBCUri;
            assert_eq!(
                ClientOptions::parse("mongodb://foo:bar@127.0.0.1:27017/?ssl=true".to_string())
                    .unwrap()
                    .tls,
                ODBCUri::new("USER=foo;PWD=bar;SERVER=127.0.0.1:27017;SSL=true")
                    .unwrap()
                    .try_into_client_options()
                    .unwrap()
                    .tls
            );
        }
    }
}
