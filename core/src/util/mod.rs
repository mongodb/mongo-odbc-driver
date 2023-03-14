mod decimal128;
use bson::doc;
use constants::SQL_ALL_TABLE_TYPES;
pub use decimal128::Decimal128Plus;
use lazy_static::lazy_static;
use mongodb::results::CollectionType;
use regex::{Regex, RegexSet, RegexSetBuilder};

pub(crate) const TABLE: &str = "TABLE";
pub(crate) const COLLECTION: &str = "collection";
pub(crate) const TIMESERIES: &str = "timeseries";
pub(crate) const VIEW: &str = "view";

lazy_static! {
    pub(crate) static ref TABLE_VALUES: RegexSet = RegexSetBuilder::new(["^table$", "^\'table\'$"])
        .case_insensitive(true)
        .build()
        .unwrap();
    pub(crate) static ref VIEW_VALUES: RegexSet = RegexSetBuilder::new(["^view$", "^\'view\'$"])
        .case_insensitive(true)
        .build()
        .unwrap();
}

// Converts SQL pattern characters (% and _) into proper regex patterns.
// SQL-1308: Handle SQL_ATTR_METADATA_ID
// SQL-1060: Improve SQL-to-Rust regex pattern method
// Returns regex for a filter
pub(crate) fn to_name_regex(filter: &str) -> Option<Regex> {
    match filter {
        "%" => None,
        _ => {
            if filter.is_empty() {
                return None;
            }
            let filter = "^".to_owned() + &filter.replace('%', ".*").replace('_', ".") + "$";

            Some(Regex::new(&filter).unwrap())
        }
    }
}

pub(crate) fn is_match(name: &str, filter: &Option<Regex>) -> bool {
    match filter {
        Some(regex) => regex.is_match(name),
        None => true,
    }
}

// Create the list of Collection types to filter on
pub(crate) fn table_type_filter_to_vec(table_type: &str) -> Option<Vec<CollectionType>> {
    return match table_type {
        SQL_ALL_TABLE_TYPES => None,
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
    };
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
    use super::{is_match, to_name_regex};

    #[test]
    fn test_to_name_regex() {
        assert!(to_name_regex("%").is_none());
        assert!(to_name_regex("").is_none());
        assert!(to_name_regex("filter").is_some());
        assert!(to_name_regex("customers").is_some());
    }

    #[test]
    fn test_is_match() {
        assert!(is_match("filter", &to_name_regex("%")));
        assert!(is_match("filter", &to_name_regex("filter")));
        assert!(is_match("downtimes", &to_name_regex("downtimes")));
        assert!(is_match("status", &to_name_regex("status")));
        assert!(is_match("customer_sales", &to_name_regex("customer_sales")));
        assert!(is_match("field_name", &to_name_regex("field_name")));

        assert!(!is_match("filter", &to_name_regex("filt")));
        assert!(!is_match("downtimestatus", &to_name_regex("downtimes")));
        assert!(!is_match("downtimestatus", &to_name_regex("status")));
    }
}
