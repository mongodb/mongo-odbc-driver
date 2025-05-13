#include "common.h"

OdbcHandles init_odbc(SQLSMALLINT odbc_version) {
    OdbcHandles handles = {SQL_NULL_HANDLE, SQL_NULL_HANDLE, SQL_NULL_HANDLE, SQL_SUCCESS};
    
    handles.last_result = SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &handles.env_handle);
    if (!is_success(handles.last_result)) {
        fprintf(stderr, "Failed to allocate environment handle.\n");
        return handles;
    }
    
    handles.last_result = SQLSetEnvAttr(handles.env_handle, SQL_ATTR_ODBC_VERSION, 
                                      (SQLPOINTER)(long)odbc_version, 0);
    if (!is_success(handles.last_result)) {
        fprintf(stderr, "Failed to set ODBC version.\n");
        print_odbc_errors(SQL_HANDLE_ENV, handles.env_handle);
        SQLFreeHandle(SQL_HANDLE_ENV, handles.env_handle);
        handles.env_handle = SQL_NULL_HANDLE;
        return handles;
    }
    
    handles.last_result = SQLAllocHandle(SQL_HANDLE_DBC, handles.env_handle, &handles.dbc_handle);
    if (!is_success(handles.last_result)) {
        fprintf(stderr, "Failed to allocate connection handle.\n");
        print_odbc_errors(SQL_HANDLE_ENV, handles.env_handle);
        SQLFreeHandle(SQL_HANDLE_ENV, handles.env_handle);
        handles.env_handle = SQL_NULL_HANDLE;
        return handles;
    }
    
    return handles;
}

SQLRETURN connect_to_mongodb(OdbcHandles *handles, const char *conn_str) {
    SQLCHAR out_conn_str[1024];
    SQLSMALLINT out_conn_str_len;
    
    handles->last_result = SQLSetConnectAttr(handles->dbc_handle, SQL_ATTR_LOGIN_TIMEOUT, 
                                           (SQLPOINTER)15, SQL_IS_INTEGER);
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to set login timeout.\n");
        print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
        return handles->last_result;
    }
    
    handles->last_result = SQLDriverConnect(
        handles->dbc_handle,
        NULL,                      // No window handle
        (SQLCHAR*)conn_str,        // Connection string
        SQL_NTS,                   // Connection string is null-terminated
        out_conn_str,              // Output connection string buffer
        sizeof(out_conn_str),      // Size of output buffer
        &out_conn_str_len,         // Length of output connection string
        SQL_DRIVER_NOPROMPT        // No prompt
    );
    
    if (is_success(handles->last_result)) {
        printf("Successfully connected to MongoDB!\n");
        printf("Output connection string: %s\n", out_conn_str);
    } else {
        fprintf(stderr, "Failed to connect to MongoDB.\n");
        print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
    }
    
    return handles->last_result;
}

void get_odbc_error(SQLSMALLINT handle_type, SQLHANDLE handle, OdbcError *error) {
    SQLSMALLINT i = 1;
    SQLRETURN ret;
    
    ret = SQLGetDiagRec(handle_type, handle, i, 
                      error->sql_state, &error->native_error, 
                      error->message, sizeof(error->message), 
                      &error->message_len);
                      
    if (!SQL_SUCCEEDED(ret)) {
        strcpy((char*)error->sql_state, "00000");
        error->native_error = 0;
        strcpy((char*)error->message, "No error information available");
        error->message_len = strlen((char*)error->message);
    }
}

void print_odbc_errors(SQLSMALLINT handle_type, SQLHANDLE handle) {
    SQLSMALLINT i = 0;
    SQLINTEGER native;
    SQLCHAR state[7];
    SQLCHAR message[SQL_MAX_MESSAGE_LENGTH + 1];
    SQLSMALLINT len;
    SQLRETURN ret;
    
    fprintf(stderr, "\n--- ODBC Errors ---\n");
    
    do {
        ret = SQLGetDiagRec(handle_type, handle, ++i, state, &native, 
                          message, sizeof(message), &len);
        if (SQL_SUCCEEDED(ret)) {
            fprintf(stderr, "[%s] (%d) %s\n", state, native, message);
        }
    } while (SQL_SUCCEEDED(ret));
    
    fprintf(stderr, "-------------------\n");
}

int is_success(SQLRETURN result) {
    return (result == SQL_SUCCESS || result == SQL_SUCCESS_WITH_INFO);
}

void cleanup_odbc(OdbcHandles *handles) {
    if (handles->stmt_handle != SQL_NULL_HANDLE) {
        SQLFreeHandle(SQL_HANDLE_STMT, handles->stmt_handle);
        handles->stmt_handle = SQL_NULL_HANDLE;
    }
    
    if (handles->dbc_handle != SQL_NULL_HANDLE) {
        SQLDisconnect(handles->dbc_handle);
        SQLFreeHandle(SQL_HANDLE_DBC, handles->dbc_handle);
        handles->dbc_handle = SQL_NULL_HANDLE;
    }
    
    if (handles->env_handle != SQL_NULL_HANDLE) {
        SQLFreeHandle(SQL_HANDLE_ENV, handles->env_handle);
        handles->env_handle = SQL_NULL_HANDLE;
    }
}

SQLRETURN execute_query(OdbcHandles *handles, const char *query) {
    if (handles->stmt_handle == SQL_NULL_HANDLE) {
        handles->last_result = SQLAllocHandle(SQL_HANDLE_STMT, handles->dbc_handle, &handles->stmt_handle);
        if (!is_success(handles->last_result)) {
            fprintf(stderr, "Failed to allocate statement handle.\n");
            print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            return handles->last_result;
        }
    }
    
    handles->last_result = SQLExecDirect(handles->stmt_handle, (SQLCHAR*)query, SQL_NTS);
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Query execution failed: %s\n", query);
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    
    return handles->last_result;
}

SQLRETURN get_tables_metadata(OdbcHandles *handles, 
                             const char *catalog, 
                             const char *schema, 
                             const char *table,
                             const char *table_type) {
    if (handles->stmt_handle == SQL_NULL_HANDLE) {
        handles->last_result = SQLAllocHandle(SQL_HANDLE_STMT, handles->dbc_handle, &handles->stmt_handle);
        if (!is_success(handles->last_result)) {
            fprintf(stderr, "Failed to allocate statement handle.\n");
            print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            return handles->last_result;
        }
    }
    
    handles->last_result = SQLTables(
        handles->stmt_handle,
        catalog ? (SQLCHAR*)catalog : NULL, catalog ? SQL_NTS : 0,
        schema ? (SQLCHAR*)schema : NULL, schema ? SQL_NTS : 0,
        table ? (SQLCHAR*)table : NULL, table ? SQL_NTS : 0,
        table_type ? (SQLCHAR*)table_type : NULL, table_type ? SQL_NTS : 0
    );
    
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "SQLTables failed.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    
    return handles->last_result;
}

SQLRETURN get_columns_metadata(OdbcHandles *handles,
                              const char *catalog,
                              const char *schema,
                              const char *table,
                              const char *column) {
    if (handles->stmt_handle == SQL_NULL_HANDLE) {
        handles->last_result = SQLAllocHandle(SQL_HANDLE_STMT, handles->dbc_handle, &handles->stmt_handle);
        if (!is_success(handles->last_result)) {
            fprintf(stderr, "Failed to allocate statement handle.\n");
            print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            return handles->last_result;
        }
    }
    
    handles->last_result = SQLColumns(
        handles->stmt_handle,
        catalog ? (SQLCHAR*)catalog : NULL, catalog ? SQL_NTS : 0,
        schema ? (SQLCHAR*)schema : NULL, schema ? SQL_NTS : 0,
        table ? (SQLCHAR*)table : NULL, table ? SQL_NTS : 0,
        column ? (SQLCHAR*)column : NULL, column ? SQL_NTS : 0
    );
    
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "SQLColumns failed.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    
    return handles->last_result;
}

SQLRETURN set_connection_attr(OdbcHandles *handles, 
                             SQLINTEGER attribute, 
                             SQLPOINTER value, 
                             SQLINTEGER string_length) {
    handles->last_result = SQLSetConnectAttr(
        handles->dbc_handle,
        attribute,
        value,
        string_length
    );
    
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to set connection attribute %ld.\n", (long)attribute);
        print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
    }
    
    return handles->last_result;
}

SQLRETURN get_connection_attr(OdbcHandles *handles,
                             SQLINTEGER attribute,
                             SQLPOINTER value_ptr,
                             SQLINTEGER buffer_length,
                             SQLINTEGER *string_length_ptr) {
    handles->last_result = SQLGetConnectAttr(
        handles->dbc_handle,
        attribute,
        value_ptr,
        buffer_length,
        string_length_ptr
    );
    
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to get connection attribute %ld.\n", (long)attribute);
        print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
    }
    
    return handles->last_result;
}

SQLRETURN get_connection_info(OdbcHandles *handles,
                             SQLUSMALLINT info_type,
                             SQLPOINTER info_value_ptr,
                             SQLSMALLINT buffer_length,
                             SQLSMALLINT *string_length_ptr) {
    handles->last_result = SQLGetInfo(
        handles->dbc_handle,
        info_type,
        info_value_ptr,
        buffer_length,
        string_length_ptr
    );
    
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to get information type %d.\n", info_type);
        print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
    }
    
    return handles->last_result;
}
