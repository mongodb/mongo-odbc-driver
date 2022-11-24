mod decimal128;
use bson::{doc, Document};
pub use decimal128::Decimal128Plus;

// Replaces SQL wildcard characters with associated regex
// Returns a doc applying filter to name
// SQL-1060: Improve SQL-to-Rust regex pattern method
pub(crate) fn to_name_regex(filter: &str) -> Document {
    let regex_filter = filter.replace('%', ".*").replace('_', ".");
    doc! { "name": { "$regex": regex_filter } }
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
