/**
 * @file mongo_odbc_ffi.h
 * @brief FFI interface for MongoDB ODBC driver core functionality
 */

#ifndef MONGO_ODBC_FFI_H
#define MONGO_ODBC_FFI_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Error codes for MongoDB ODBC FFI functions
 */
typedef enum {
    MONGO_ODBC_SUCCESS = 0,
    MONGO_ODBC_CONNECTION_FAILED = 1,
    MONGO_ODBC_QUERY_PREPARATION_FAILED = 2,
    MONGO_ODBC_QUERY_EXECUTION_FAILED = 3,
    MONGO_ODBC_INVALID_PARAMETER = 4,
    MONGO_ODBC_INVALID_CURSOR_STATE = 5,
    MONGO_ODBC_OUT_OF_MEMORY = 6,
    MONGO_ODBC_UNKNOWN_ERROR = 7
} MongoOdbcErrorCode;

/**
 * Opaque handle type for MongoDB connections
 */
typedef struct ConnectionHandle ConnectionHandle;

/**
 * Opaque handle type for MongoDB statements
 */
typedef struct StatementHandle StatementHandle;

/**
 * Create a connection to MongoDB
 *
 * @param connection_string ODBC-style connection string
 * @param error_code Pointer to store error code (can be NULL)
 * @return Connection handle or NULL on error
 */
ConnectionHandle* mongo_odbc_connect(const char* connection_string, MongoOdbcErrorCode* error_code);

/**
 * Free a MongoDB connection
 *
 * @param handle Connection handle to free
 */
void mongo_odbc_free_connection(ConnectionHandle* handle);

/**
 * Prepare a MongoDB query
 *
 * @param connection_handle Valid connection handle
 * @param query SQL query string
 * @param error_code Pointer to store error code (can be NULL)
 * @return Statement handle or NULL on error
 */
StatementHandle* mongo_odbc_prepare_query(const ConnectionHandle* connection_handle, const char* query, MongoOdbcErrorCode* error_code);

/**
 * Execute a prepared statement
 *
 * @param connection_handle Valid connection handle
 * @param statement_handle Valid statement handle
 * @param error_code Pointer to store error code (can be NULL)
 * @return true on success, false on failure
 */
bool mongo_odbc_execute_statement(const ConnectionHandle* connection_handle, StatementHandle* statement_handle, MongoOdbcErrorCode* error_code);

/**
 * Free a statement
 *
 * @param handle Statement handle to free
 */
void mongo_odbc_free_statement(StatementHandle* handle);

/**
 * Fetch the next row from a result set
 *
 * @param statement_handle Valid statement handle
 * @param error_code Pointer to store error code (can be NULL)
 * @return true if a row was fetched, false otherwise
 */
bool mongo_odbc_fetch(StatementHandle* statement_handle, MongoOdbcErrorCode* error_code);

/**
 * Get error message for a given error code
 *
 * @param error_code Error code
 * @return Error message string (do not free)
 */
const char* mongo_odbc_get_error_message(MongoOdbcErrorCode error_code);

#ifdef __cplusplus
}
#endif

#endif /* MONGO_ODBC_FFI_H */
