#[cfg(target_os = "windows")]
mod dsn;
#[cfg(target_os = "windows")]
pub mod odbcinst;
#[cfg(target_os = "windows")]
pub use dsn::DSNOpts;
