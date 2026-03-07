//! Retry wrapper for MongoDB `run_command` operations
//!
//! This module provides a retry mechanism for MongoDB `run_command` operations
//! that encounter `ConnectionPoolCleared` errors. The retry logic uses exponential backoff
//! to handle transient connection pool issues.
//!
//! # Example Usage
//!
//! ```no_run
//! use mongodb::{bson::doc, Client};
//! use mongo_odbc_core::run_command::run_command_with_retry;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::with_uri_str("mongodb://localhost:27017").await?;
//! let db = client.database("admin");
//!
//! // With retries (max 3 retries, exponential backoff)
//! let cmd = doc! { "buildInfo": 1 };
//! let result = run_command_with_retry(&db, cmd).await?;
//!
//! // Without retries (use plain run_command)
//! let result = db.run_command(doc! { "ping": 1 }).await?;
//! # Ok(())
//! # }
//! ```

use mongodb::{
    bson::Document,
    error::ErrorKind,
    Database,
};
use std::time::Duration;

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 100;

/// Check if an error is a ConnectionPoolCleared error
fn is_connection_pool_cleared_error(error: &mongodb::error::Error) -> bool {
    matches!(error.kind.as_ref(), ErrorKind::ConnectionPoolCleared { .. })
}

/// Execute a database command with retry on ConnectionPoolCleared errors
///
/// This function wraps the MongoDB `run_command` method and provides retry logic
/// for `ConnectionPoolCleared` errors. It will retry up to `MAX_RETRIES` times (3)
/// with exponential backoff starting at `BASE_DELAY_MS` (100ms).
///
/// If you don't want retry behavior, use `db.run_command()` directly instead.
///
/// # Arguments
///
/// * `db` - The MongoDB database instance
/// * `command` - The BSON document representing the command to execute
///
/// # Returns
///
/// Returns a `Result` containing the command response document or an error
///
/// # Retry Behavior
///
/// - Only retries on `ConnectionPoolCleared` errors
/// - Maximum 3 retry attempts
/// - Exponential backoff: 200ms, 400ms, 800ms
/// - All other errors are returned immediately
///
/// # Example
///
/// ```no_run
/// use mongodb::{bson::doc, Database};
/// use mongo_odbc_core::run_command::run_command_with_retry;
///
/// async fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
///     let cmd = doc! { "buildInfo": 1 };
///
///     // With retries (use this function)
///     let result = run_command_with_retry(db, cmd.clone()).await?;
///
///     // Without retries (use plain run_command)
///     let result = db.run_command(cmd).await?;
///
///     Ok(())
/// }
/// ```
pub async fn run_command_with_retry(
    db: &Database,
    command: Document,
) -> std::result::Result<Document, mongodb::error::Error> {
    let mut attempt = 0;
    loop {
        match db.run_command(command.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                // Check if we should retry
                if attempt >= MAX_RETRIES || !is_connection_pool_cleared_error(&e) {
                    return Err(e);
                }

                // Calculate exponential backoff delay: base_delay * 2^attempt
                let delay_ms = BASE_DELAY_MS * (1 << attempt);
                let delay = Duration::from_millis(delay_ms);

                log::warn!(
                    "ConnectionPoolCleared error encountered. Retrying attempt {}/{} after {:?}",
                    attempt + 1,
                    MAX_RETRIES,
                    delay
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

