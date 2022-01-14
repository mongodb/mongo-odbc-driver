pub mod collections;
pub mod conn;
pub mod databases;
mod err;
pub mod fields;
pub mod query;
pub mod stmt;
pub use err::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // no-op
    }
}
