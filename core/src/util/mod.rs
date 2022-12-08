mod decimal128;
use bson::{doc, Bson, Document};
use constants::SQL_ALL_TABLE_TYPES;
pub use decimal128::Decimal128Plus;
use lazy_static::lazy_static;
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

// Converts % pattern character into proper regex patterns.
// SQL-1060: Improve SQL-to-Rust regex pattern method
fn pattern_to_regex(filter: &str) -> String {
    filter.replace('%', ".*").replace('_', ".")
}

// Returns a doc applying filter to name
pub(crate) fn to_name_regex_doc(filter: &str) -> Document {
    let regex_filter = pattern_to_regex(filter);
    doc! { "name": { "$regex": regex_filter } }
}

// Returns regex for a filter
pub(crate) fn to_name_regex(filter: &str) -> Regex {
    let regex_filter = pattern_to_regex(filter);
    // If this ever fails it reflects a failure in pattern_to_regex, so this unwrap can stay
    Regex::new(&regex_filter).unwrap()
}

// Iterates through the table types and adds the corresponding type to the filter document.
pub(crate) fn add_table_type_filter(table_type: &str, mut filter: Document) -> Document {
    let mut table_type_filters: Vec<Bson> = Vec::new();
    let table_type_entries = table_type
        .split(',')
        .map(|attr| attr.trim())
        .collect::<Vec<&str>>();
    for table_type_entry in &table_type_entries {
        if SQL_ALL_TABLE_TYPES.to_string().eq(table_type_entry) {
            // No need to add a 'type' filter
            return filter;
        } else if TABLE_VALUES.is_match(table_type_entry) {
            // Collection and Timeseries types are mapped to table
            table_type_filters.push(Bson::String(COLLECTION.to_string()));
            table_type_filters.push(Bson::String(TIMESERIES.to_string()));
        } else if VIEW_VALUES.is_match(table_type_entry) {
            table_type_filters.push(Bson::String(VIEW.to_string()));
        }
    }
    filter.insert("type", doc! {"$in": Bson::Array(table_type_filters) });
    filter
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
