pub(crate) mod definitions;
pub(crate) mod errors;
mod functions;
pub(crate) mod odbc_uri;
pub use functions::*;
mod data;

#[cfg(test)]
mod col_attr_tests;
#[cfg(test)]
mod data_tests;
#[cfg(test)]
mod driver_connect_tests;
#[cfg(test)]
mod env_attr_tests;
#[cfg(test)]
mod get_diag_rec_tests;
#[cfg(test)]
mod stmt_attr_tests;

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
				$val
			},)*
		].into_iter())
	};
}
