#ifndef MONGODB_ODBC_COMMON_H
#define MONGODB_ODBC_COMMON_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sql.h>
#include <sqlext.h>

// ODBC Handle management
typedef struct {
    SQLHENV env_handle;
    SQLHDBC dbc_handle;
    SQLHSTMT stmt_handle;
    SQLRETURN last_result;
} OdbcHandles;

// Error information structure
typedef struct {
    SQLCHAR sql_state[6];
    SQLINTEGER native_error;
    SQLCHAR message[SQL_MAX_MESSAGE_LENGTH + 1];
    SQLSMALLINT message_len;
} OdbcError;

// MongoDB-specific data type constants
#define MONGODB_TYPE_OBJECTID "ObjectId"
#define MONGODB_TYPE_DOUBLE "double"
#define MONGODB_TYPE_STRING "string"
#define MONGODB_TYPE_OBJECT "object"
#define MONGODB_TYPE_ARRAY "array"
#define MONGODB_TYPE_BINDATA "binData"
#define MONGODB_TYPE_UNDEFINED "undefined"
#define MONGODB_TYPE_BOOL "bool"
#define MONGODB_TYPE_DATE "date"
#define MONGODB_TYPE_NULL "null"
#define MONGODB_TYPE_REGEX "regex"
#define MONGODB_TYPE_INT "int"
#define MONGODB_TYPE_TIMESTAMP "timestamp"
#define MONGODB_TYPE_LONG "long"

// Initialize ODBC environment and allocate handles
OdbcHandles init_odbc(SQLSMALLINT odbc_version);

// Connect to MongoDB using connection string
SQLRETURN connect_to_mongodb(OdbcHandles *handles, const char *conn_str);

// Get comprehensive error information
void get_odbc_error(SQLSMALLINT handle_type, SQLHANDLE handle, OdbcError *error);

// Print all available error information for a handle
void print_odbc_errors(SQLSMALLINT handle_type, SQLHANDLE handle);

// Check if a result code indicates success
int is_success(SQLRETURN result);

// Free all allocated handles and resources
void cleanup_odbc(OdbcHandles *handles);

// Execute a SQL statement and check for errors
SQLRETURN execute_query(OdbcHandles *handles, const char *query);

// Get metadata about available tables/collections
SQLRETURN get_tables_metadata(OdbcHandles *handles, 
                             const char *catalog, 
                             const char *schema, 
                             const char *table,
                             const char *table_type);

// Get column metadata for a table/collection
SQLRETURN get_columns_metadata(OdbcHandles *handles,
                              const char *catalog,
                              const char *schema,
                              const char *table,
                              const char *column);

// Set connection attributes
SQLRETURN set_connection_attr(OdbcHandles *handles, 
                             SQLINTEGER attribute, 
                             SQLPOINTER value, 
                             SQLINTEGER string_length);

// Get connection attributes
SQLRETURN get_connection_attr(OdbcHandles *handles,
                             SQLINTEGER attribute,
                             SQLPOINTER value_ptr,
                             SQLINTEGER buffer_length,
                             SQLINTEGER *string_length_ptr);

// Get connection information
SQLRETURN get_connection_info(OdbcHandles *handles,
                             SQLUSMALLINT info_type,
                             SQLPOINTER info_value_ptr,
                             SQLSMALLINT buffer_length,
                             SQLSMALLINT *string_length_ptr);

#endif // MONGODB_ODBC_COMMON_H
