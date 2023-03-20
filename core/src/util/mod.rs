mod decimal128;
use bson::doc;
use constants::SQL_ALL_TABLE_TYPES;
pub use decimal128::Decimal128Plus;
use fancy_regex::Regex as FancyRegex;
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
    static ref NON_ESCAPED_UNDERSCORE: FancyRegex = FancyRegex::new(r"(?<!\\)_").unwrap();
    static ref NON_ESCAPED_PERCENT: FancyRegex = FancyRegex::new(r"(?<!\\)%").unwrap();
    static ref ESCAPED_UNDERSCORE: FancyRegex = FancyRegex::new(r"\\_").unwrap();
    static ref ESCAPED_PERCENT: FancyRegex = FancyRegex::new(r"\\%").unwrap();
}

// Converts SQL pattern characters (% and _) into proper regex patterns.
// SQL-1308: Handle SQL_ATTR_METADATA_ID
// SQL-1060: Improve SQL-to-Rust regex pattern method
// Returns regex for a filter
pub(crate) fn to_name_regex(filter: &str) -> Option<Regex> {
    match filter {
        "%" | "" => None,
        _ => {
            let filter = NON_ESCAPED_UNDERSCORE.replace_all(&filter, ".");
            let filter = NON_ESCAPED_PERCENT.replace_all(&filter, ".*");
            let filter = ESCAPED_UNDERSCORE.replace_all(&filter, "_");
            let filter = ESCAPED_PERCENT.replace_all(&filter, "%");
            let filter = if filter.starts_with("^") || filter.ends_with("$") {
                filter.to_string()
            } else {
                format!("^{}$", filter)
            };

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
    }

    #[test]
    fn test_is_positive_match() {
        assert!(is_match("filter", &to_name_regex("%")));
        assert!(is_match("filter", &to_name_regex("filter")));
        assert!(is_match("downtimes", &to_name_regex("downtimes")));
        assert!(is_match("customer_sales", &to_name_regex("customer_sales")));
        assert!(is_match("myiphone", &to_name_regex("my_phone")));
        assert!(is_match("conversions2022", &to_name_regex("conversions%")));
        assert!(is_match("integration_test", &to_name_regex("%test")));
    }

    #[test]
    fn test_is_negative_match() {
        assert!(!is_match("filter", &to_name_regex("filt")));
        assert!(!is_match("filter", &to_name_regex(r"filt_er")));
        assert!(!is_match("downtimestatus", &to_name_regex("downtimes")));
        assert!(!is_match("downtimestatus", &to_name_regex("status")));
        assert!(!is_match("integration_test_2", &to_name_regex("%test")));
    }

    #[test]
    fn test_escaped_chars() {
        assert!(is_match("my_phone", &to_name_regex(r"my\_phone")));
        assert!(!is_match("myiphone", &to_name_regex(r"my\_phone")));
        assert!(is_match(
            "conversion%2022",
            &to_name_regex(r"conversion\%2022")
        ));
        assert!(!is_match(
            "conversions2022",
            &to_name_regex(r"conversion\%2022")
        ));
    }
}
