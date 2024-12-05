use constants::SQL_ALL_TABLE_TYPES;
use fancy_regex::Regex as FancyRegex;
use lazy_static::lazy_static;
use mongodb::results::CollectionType;
use regex::{Regex, RegexSet, RegexSetBuilder};

pub mod test_connection;

pub(crate) const TABLE: &str = "TABLE";
pub(crate) const COLLECTION: &str = "collection";
pub(crate) const TIMESERIES: &str = "timeseries";
pub(crate) const VIEW: &str = "view";
pub(crate) const DISALLOWED_DB_NAMES: [&str; 4] = ["admin", "config", "local", "system"];
pub(crate) const DISALLOWED_COLLECTION_NAMES: [&str; 6] = [
    "system.namespaces",
    "system.indexes",
    "system.profile",
    "system.js",
    "system.views",
    "__sql_schemas",
];

lazy_static! {
    pub(crate) static ref TABLE_VALUES: RegexSet = RegexSetBuilder::new(["^table$", "^\'table\'$"])
        .case_insensitive(true)
        .build()
        .unwrap();
    pub(crate) static ref VIEW_VALUES: RegexSet = RegexSetBuilder::new(["^view$", "^\'view\'$"])
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref NON_ESCAPED_UNDERSCORE: FancyRegex = FancyRegex::new(r"(?<!\\\\)_").unwrap();
    static ref NON_ESCAPED_PERCENT: FancyRegex = FancyRegex::new(r"(?<!\\\\)%").unwrap();
}

// Converts SQL pattern characters (% and _) into proper regex patterns.
// SQL-1308: Handle SQL_ATTR_METADATA_ID
// Returns regex for a filter
pub(crate) fn to_name_regex(filter: &str) -> Option<Regex> {
    match filter {
        "%" | "" => None,
        _ => {
            let filter = regex::escape(filter);
            let filter = NON_ESCAPED_UNDERSCORE.replace_all(&filter, ".");
            let filter = NON_ESCAPED_PERCENT.replace_all(&filter, ".*");
            let filter = &filter.replace("\\\\_", "_").replace("\\\\%", "%");
            Some(Regex::new(&format!("^{filter}$")).unwrap())
        }
    }
}

/// is_match compares `name` to `filter` either directly or via regex, depending on
/// the value `accept_search_patterns`. Empty strings for filters will match everything.
pub(crate) fn is_match(name: &str, filter: &str, accept_search_patterns: bool) -> bool {
    match accept_search_patterns {
        false => filter.is_empty() || name == filter,
        true => match to_name_regex(filter) {
            Some(regex) => regex.is_match(name),
            None => true,
        },
    }
}

// Create the list of Collection types to filter on
pub(crate) fn table_type_filter_to_vec(table_type: &str) -> Option<Vec<CollectionType>> {
    match table_type {
        SQL_ALL_TABLE_TYPES | "" => None,
        _ => {
            let table_type_entries = table_type
                .split(',')
                .map(|attr| attr.trim())
                .collect::<Vec<&str>>();
            let mut table_type_filters: Vec<CollectionType> = Vec::new();
            for table_type_entry in &table_type_entries {
                if TABLE_VALUES.is_match(table_type_entry) {
                    // Collection and Timeseries types should be mapped to table
                    // The Rust driver doesn't seem to deserialize timeseries at the moment because
                    // there is no CollectionType::Timeseries
                    table_type_filters.push(CollectionType::Collection);
                } else if VIEW_VALUES.is_match(table_type_entry) {
                    table_type_filters.push(CollectionType::View);
                }
            }
            Some(table_type_filters)
        }
    }
}

#[macro_export]
macro_rules! map {
        ($($key:expr => $val:expr),* $(,)?) => {
            std::iter::Iterator::collect([
                $({
                    ($key, $val)
                },)*
            ].into_iter())
        };
    }

#[macro_export]
macro_rules! set {
        ($($val:expr),* $(,)?) => {
            std::iter::Iterator::collect([
                $({
                    ($val)
                },)*
            ].into_iter())
        };
    }

#[cfg(test)]
mod filtering {
    use super::{is_match, table_type_filter_to_vec, to_name_regex};

    mod table_type_filter_to_vec {
        use super::table_type_filter_to_vec;
        use mongodb::results::CollectionType;

        #[test]
        fn test_table_type_filter_to_vec() {
            assert_eq!(
                table_type_filter_to_vec("table"),
                Some(vec![CollectionType::Collection])
            );
            assert_eq!(
                table_type_filter_to_vec("view"),
                Some(vec![CollectionType::View])
            );
            assert_eq!(
                table_type_filter_to_vec("table,view"),
                Some(vec![CollectionType::Collection, CollectionType::View])
            );
            assert_eq!(table_type_filter_to_vec(""), None);
            assert_eq!(table_type_filter_to_vec("%"), None);
        }
    }

    mod to_name_regex {
        use super::to_name_regex;

        #[test]
        fn test_to_name_regex() {
            assert!(to_name_regex("%").is_none());
            assert!(to_name_regex("").is_none());
            assert!(to_name_regex("filter").is_some());
        }
    }

    mod is_match {
        use super::is_match;
        #[test]
        fn test_is_positive_match_literal() {
            assert!(is_match("%", "%", false));
            assert!(is_match("%test", "%test", false));
            assert!(is_match("down_times", "down_times", false));
            assert!(is_match("filter", "filter", false));
            assert!(is_match("downtimes", "downtimes", false));
            assert!(is_match("money$$bags", "money$$bags", false));
            assert!(is_match("money$.bags", "money$.bags", false));
        }

        #[test]
        fn test_is_negative_match_literal() {
            assert!(!is_match("filter", "%", false));
            assert!(!is_match("customerssales", "customer_sales", false));
            assert!(!is_match("conversions2022", "conversions%", false));
            assert!(!is_match("integration_test", "%test", false));
            assert!(!is_match("integration_test", "integrationstest", false));
        }

        #[test]
        fn test_is_positive_match_pattern() {
            assert!(is_match("filter", "%", true));
            assert!(is_match("filter", "filter", true));
            assert!(is_match("downtimes", "downtimes", true));
            assert!(is_match("customerssales", "customer_sales", true));
            assert!(is_match("myiphone", "my_phone", true));
            assert!(is_match("conversions2022", "conversions%", true));
            assert!(is_match("integration_test", "%test", true));
            assert!(is_match("money$$bags", "money$$bags", true));
            assert!(is_match("money$.bags", "money$.bags", true));
        }

        #[test]
        fn test_is_negative_match_odbc_pattern() {
            assert!(!is_match("filter", "filt", true));
            assert!(!is_match("filter", r"filt_er", true));
            assert!(!is_match("downtimestatus", "downtimes", true));
            assert!(!is_match("downtimestatus", "status", true));
            assert!(!is_match("integration_test_2", "%test", true));
            assert!(!is_match("money$$bags", "money$.bags", true));
        }

        #[test]
        fn test_escaped_chars_in_pattern() {
            assert!(is_match("my_phone", r"my\_phone", true));
            assert!(!is_match("myiphone", r"my\_phone", true));
            assert!(is_match("conversion%2022", r"conversion\%2022", true));
            assert!(!is_match("conversions2022", r"conversion\%2022", true));
        }
    }
}
