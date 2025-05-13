/**
 * @file hybrid_mongo_connector.c
 * @brief Example demonstrating hybrid use of ODBC API and direct MongoDB ODBC driver core API
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sql.h>
#include <sqlext.h>
#include "mongo_odbc_ffi.h"

#define SUCCESS(ret) ((ret) == SQL_SUCCESS || (ret) == SQL_SUCCESS_WITH_INFO)

void print_odbc_error(SQLSMALLINT handle_type, SQLHANDLE handle) {
    SQLCHAR sql_state[6], message[SQL_MAX_MESSAGE_LENGTH];
    SQLINTEGER native_error;
    SQLSMALLINT message_len;
    SQLRETURN ret;
    
    if (SQL_SUCCESS == (ret = SQLGetDiagRec(handle_type, handle, 1, sql_state, 
                                          &native_error, message, sizeof(message), &message_len))) {
        printf("ODBC Error: [%s] %s (Native error: %d)\n", sql_state, message, (int)native_error);
    }
}

void print_direct_error(MongoOdbcErrorCode error_code) {
    const char* message = mongo_odbc_get_error_message(error_code);
    printf("Direct API Error: %s (code %d)\n", message, error_code);
}

int main(int argc, char** argv) {
    const char* connection_string = "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/";
    const char* query = "SELECT * FROM system.version";
    SQLRETURN ret;
    MongoOdbcErrorCode error_code = MONGO_ODBC_SUCCESS;
    
    if (argc > 1) {
        connection_string = argv[1];
    }
    if (argc > 2) {
        query = argv[2];
    }
    
    printf("Connection string: %s\n", connection_string);
    printf("Query: %s\n", query);
    
    printf("\n=== Using standard ODBC API ===\n");
    
    SQLHENV env_handle = SQL_NULL_HANDLE;
    SQLHDBC dbc_handle = SQL_NULL_HANDLE;
    SQLHSTMT stmt_handle = SQL_NULL_HANDLE;
    
    ret = SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &env_handle);
    if (!SUCCESS(ret)) {
        printf("Failed to allocate environment handle\n");
        return 1;
    }
    
    ret = SQLSetEnvAttr(env_handle, SQL_ATTR_ODBC_VERSION, (SQLPOINTER)SQL_OV_ODBC3, SQL_IS_INTEGER);
    if (!SUCCESS(ret)) {
        printf("Failed to set ODBC version\n");
        SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
        return 1;
    }
    
    ret = SQLAllocHandle(SQL_HANDLE_DBC, env_handle, &dbc_handle);
    if (!SUCCESS(ret)) {
        printf("Failed to allocate connection handle\n");
        SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
        return 1;
    }
    
    SQLCHAR conn_out[1024];
    SQLSMALLINT conn_out_len;
    ret = SQLDriverConnect(dbc_handle, NULL, (SQLCHAR*)connection_string, SQL_NTS, 
                          conn_out, sizeof(conn_out), &conn_out_len, SQL_DRIVER_NOPROMPT);
    if (!SUCCESS(ret)) {
        printf("Failed to connect using ODBC API\n");
        print_odbc_error(SQL_HANDLE_DBC, dbc_handle);
        SQLFreeHandle(SQL_HANDLE_DBC, dbc_handle);
        SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
        return 1;
    }
    printf("Connected successfully via ODBC API\n");
    
    ret = SQLAllocHandle(SQL_HANDLE_STMT, dbc_handle, &stmt_handle);
    if (!SUCCESS(ret)) {
        printf("Failed to allocate statement handle\n");
        SQLDisconnect(dbc_handle);
        SQLFreeHandle(SQL_HANDLE_DBC, dbc_handle);
        SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
        return 1;
    }
    
    ret = SQLExecDirect(stmt_handle, (SQLCHAR*)query, SQL_NTS);
    if (!SUCCESS(ret)) {
        printf("Failed to execute query via ODBC API\n");
        print_odbc_error(SQL_HANDLE_STMT, stmt_handle);
        SQLFreeHandle(SQL_HANDLE_STMT, stmt_handle);
        SQLDisconnect(dbc_handle);
        SQLFreeHandle(SQL_HANDLE_DBC, dbc_handle);
        SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
        return 1;
    }
    printf("Query executed successfully via ODBC API\n");
    
    printf("\nODBC API Results:\n");
    int row_count = 0;
    while (SQL_SUCCESS == SQLFetch(stmt_handle)) {
        row_count++;
        printf("Row %d fetched via ODBC API\n", row_count);
        
    }
    
    printf("\nTotal rows via ODBC API: %d\n", row_count);
    
    SQLFreeHandle(SQL_HANDLE_STMT, stmt_handle);
    SQLDisconnect(dbc_handle);
    SQLFreeHandle(SQL_HANDLE_DBC, dbc_handle);
    SQLFreeHandle(SQL_HANDLE_ENV, env_handle);
    
    printf("\n=== Using direct MongoDB ODBC core API ===\n");
    
    printf("Connecting to MongoDB via direct API...\n");
    ConnectionHandle* connection = mongo_odbc_connect(connection_string, &error_code);
    if (!connection) {
        print_direct_error(error_code);
        return 1;
    }
    printf("Connected successfully via direct API\n");
    
    printf("Preparing query via direct API...\n");
    StatementHandle* statement = mongo_odbc_prepare_query(connection, query, &error_code);
    if (!statement) {
        print_direct_error(error_code);
        mongo_odbc_free_connection(connection);
        return 1;
    }
    printf("Query prepared successfully via direct API\n");
    
    printf("Executing query via direct API...\n");
    bool success = mongo_odbc_execute_statement(connection, statement, &error_code);
    if (!success) {
        print_direct_error(error_code);
        mongo_odbc_free_statement(statement);
        mongo_odbc_free_connection(connection);
        return 1;
    }
    printf("Query executed successfully via direct API\n");
    
    printf("\nDirect API Results:\n");
    row_count = 0;
    while (mongo_odbc_fetch(statement, &error_code)) {
        row_count++;
        printf("Row %d fetched via direct API\n", row_count);
        
    }
    
    if (error_code != MONGO_ODBC_SUCCESS) {
        print_direct_error(error_code);
    }
    
    printf("\nTotal rows via direct API: %d\n", row_count);
    
    mongo_odbc_free_statement(statement);
    mongo_odbc_free_connection(connection);
    
    printf("All connections closed\n");
    
    return 0;
}
