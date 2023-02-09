pub(crate) mod definitions;
pub(crate) mod diag;
pub(crate) mod errors;
mod functions;
pub use functions::*;
#[cfg(test)]
mod col_attr_describe_tests;
#[cfg(test)]
mod connect_attr_tests;
mod data;
#[cfg(test)]
mod data_tests;
#[cfg(test)]
mod env_attr_tests;
#[cfg(test)]
mod get_diag_field_tests;
#[cfg(test)]
mod get_diag_rec_tests;
#[cfg(test)]
mod get_info_tests;
#[cfg(test)]
mod get_type_info_tests;
#[cfg(test)]
mod panic_safe_exec_tests;
#[cfg(test)]
mod stmt_attr_tests;
pub(crate) mod util;

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

///
/// Adds a line in the trace formatted like this [{handle info}]{function name} - "{message to log}"]
/// The handle info provide the address of the current handle and it's parent handle.
/// For example for a connection handle : [Env_0x131904740][Conn_0x131805040]
///
#[macro_export]
macro_rules! trace_odbc {
    ($info:expr, $fct_name:expr) => {
        // No handle have been allocated yet
        let message = format!("{}:: {}", $fct_name, $info);
        dbg_write!(message);
    };
    ($mongo_handle:expr, $info:expr, $fct_name:expr) => {
        let handle_info = $mongo_handle.get_handle_info();
        let message = format!("{handle_info} {}:: {}", $fct_name, $info);
        dbg_write!(message);
    };
}

///
/// Using the given handle, error and function name, it will:
///  - Add the error information in the trace.
///    For example : 2023-02-08T17:14:05.383-08:00 odbc/src/api/functions.rs:1151 - [Env_0x130f04740][Conn_0x131a04d40][SQLDriverConnectW] - "[MongoDB][API] Missing property "Driver" or "DSN" in connection string"
///  - Add the diagnostic record to the handle
///
#[macro_export]
macro_rules! add_diag_with_function {
    ($handle:expr, $error:expr, $fct_name:expr) => {
        let err = $error;
        trace_odbc!($handle, format!("{err}"), $fct_name);
        $handle.add_diag_info($error);
    };
}
