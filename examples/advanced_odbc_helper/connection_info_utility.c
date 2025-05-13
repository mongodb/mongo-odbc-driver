#include "common.h"

void print_info_type(OdbcHandles *handles, SQLUSMALLINT info_type, const char *info_name);
void print_connection_attributes(OdbcHandles *handles);

void show_usage(const char *program_name) {
    printf("Usage: %s [connection_string]\n", program_name);
    printf("  connection_string: ODBC connection string (optional)\n\n");
    printf("Example: %s \"Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/\"\n", 
           program_name);
}

int main(int argc, char **argv) {
    const char *conn_str = (argc > 1) ? argv[1] : 
        "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/";
    
    if (argc > 1 && (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0)) {
        show_usage(argv[0]);
        return 0;
    }
    
    OdbcHandles handles = init_odbc(SQL_OV_ODBC3);
    if (handles.env_handle == SQL_NULL_HANDLE || handles.dbc_handle == SQL_NULL_HANDLE) {
        cleanup_odbc(&handles);
        return 1;
    }
    
    if (!is_success(connect_to_mongodb(&handles, conn_str))) {
        cleanup_odbc(&handles);
        return 1;
    }
    
    printf("\n--- MongoDB ODBC Connection Information ---\n\n");
    
    printf("--- Driver and DBMS Information ---\n");
    print_info_type(&handles, SQL_DRIVER_NAME, "Driver Name");
    print_info_type(&handles, SQL_DRIVER_VER, "Driver Version");
    print_info_type(&handles, SQL_DRIVER_ODBC_VER, "Driver ODBC Version");
    print_info_type(&handles, SQL_DBMS_NAME, "DBMS Name");
    print_info_type(&handles, SQL_DBMS_VER, "DBMS Version");
    
    printf("\n--- Data Source Information ---\n");
    print_info_type(&handles, SQL_DATA_SOURCE_NAME, "Data Source Name");
    print_info_type(&handles, SQL_SERVER_NAME, "Server Name");
    print_info_type(&handles, SQL_DATABASE_NAME, "Database Name");
    print_info_type(&handles, SQL_USER_NAME, "User Name");
    
    printf("\n--- Feature Support ---\n");
    print_info_type(&handles, SQL_ACCESSIBLE_TABLES, "Accessible Tables");
    print_info_type(&handles, SQL_ACCESSIBLE_PROCEDURES, "Accessible Procedures");
    print_info_type(&handles, SQL_CURSOR_COMMIT_BEHAVIOR, "Cursor Commit Behavior");
    print_info_type(&handles, SQL_CURSOR_ROLLBACK_BEHAVIOR, "Cursor Rollback Behavior");
    print_info_type(&handles, SQL_DATA_SOURCE_READ_ONLY, "Data Source Read Only");
    print_info_type(&handles, SQL_DEFAULT_TXN_ISOLATION, "Default Transaction Isolation");
    print_info_type(&handles, SQL_MULT_RESULT_SETS, "Multiple Result Sets");
    print_info_type(&handles, SQL_PROCEDURES, "Procedures");
    
    printf("\n--- SQL Support ---\n");
    print_info_type(&handles, SQL_SQL_CONFORMANCE, "SQL Conformance");
    print_info_type(&handles, SQL_EXPRESSIONS_IN_ORDERBY, "Expressions in ORDER BY");
    print_info_type(&handles, SQL_MAX_COLUMNS_IN_SELECT, "Max Columns in SELECT");
    print_info_type(&handles, SQL_MAX_TABLES_IN_SELECT, "Max Tables in SELECT");
    print_info_type(&handles, SQL_MAX_COLUMNS_IN_GROUP_BY, "Max Columns in GROUP BY");
    print_info_type(&handles, SQL_MAX_COLUMNS_IN_ORDER_BY, "Max Columns in ORDER BY");
    
    printf("\n--- Limits ---\n");
    print_info_type(&handles, SQL_MAX_COLUMN_NAME_LEN, "Max Column Name Length");
    print_info_type(&handles, SQL_MAX_CURSOR_NAME_LEN, "Max Cursor Name Length");
    print_info_type(&handles, SQL_MAX_SCHEMA_NAME_LEN, "Max Schema Name Length");
    print_info_type(&handles, SQL_MAX_TABLE_NAME_LEN, "Max Table Name Length");
    print_info_type(&handles, SQL_MAX_USER_NAME_LEN, "Max User Name Length");
    
    printf("\n--- Connection Attributes ---\n");
    print_connection_attributes(&handles);
    
    printf("\n--- MongoDB-Specific Information ---\n");
    if (handles.stmt_handle != SQL_NULL_HANDLE) {
        SQLFreeHandle(SQL_HANDLE_STMT, handles.stmt_handle);
        handles.stmt_handle = SQL_NULL_HANDLE;
    }
    
    if (is_success(execute_query(&handles, "SELECT version() AS mongodb_version"))) {
        SQLRETURN ret;
        SQLCHAR buffer[512];
        SQLLEN indicator;
        
        if ((ret = SQLFetch(handles.stmt_handle)) == SQL_SUCCESS) {
            ret = SQLGetData(handles.stmt_handle, 1, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
            if (is_success(ret) && indicator != SQL_NULL_DATA) {
                printf("%-30s: %s\n", "MongoDB Server Version", buffer);
            }
        }
        
        SQLFreeStmt(handles.stmt_handle, SQL_CLOSE);
    }
    
    if (is_success(execute_query(&handles, "SELECT buildInfo() AS build_info"))) {
        SQLRETURN ret;
        SQLCHAR buffer[2048];
        SQLLEN indicator;
        
        if ((ret = SQLFetch(handles.stmt_handle)) == SQL_SUCCESS) {
            ret = SQLGetData(handles.stmt_handle, 1, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
            if (is_success(ret) && indicator != SQL_NULL_DATA) {
                printf("%-30s: %s\n", "MongoDB Build Info", buffer);
            }
        }
        
        SQLFreeStmt(handles.stmt_handle, SQL_CLOSE);
    }
    
    cleanup_odbc(&handles);
    return 0;
}

void print_info_type(OdbcHandles *handles, SQLUSMALLINT info_type, const char *info_name) {
    SQLCHAR buffer[512];
    SQLSMALLINT buffer_len;
    SQLUINTEGER int_value;
    SQLUSMALLINT small_int_value;
    SQLRETURN ret;
    
    printf("%-30s: ", info_name);
    
    switch (info_type) {
        case SQL_DRIVER_NAME:
        case SQL_DRIVER_VER:
        case SQL_DRIVER_ODBC_VER:
        case SQL_DBMS_NAME:
        case SQL_DBMS_VER:
        case SQL_DATA_SOURCE_NAME:
        case SQL_SERVER_NAME:
        case SQL_DATABASE_NAME:
        case SQL_USER_NAME:
            ret = SQLGetInfo(handles->dbc_handle, info_type, buffer, sizeof(buffer), &buffer_len);
            if (is_success(ret)) {
                printf("%s\n", buffer);
            } else {
                printf("Error retrieving information\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
            break;
            
        case SQL_MAX_COLUMNS_IN_SELECT:
        case SQL_MAX_TABLES_IN_SELECT:
        case SQL_MAX_COLUMNS_IN_GROUP_BY:
        case SQL_MAX_COLUMNS_IN_ORDER_BY:
        case SQL_DEFAULT_TXN_ISOLATION:
            ret = SQLGetInfo(handles->dbc_handle, info_type, &int_value, sizeof(int_value), NULL);
            if (is_success(ret)) {
                if (info_type == SQL_DEFAULT_TXN_ISOLATION) {
                    if (int_value == SQL_TXN_READ_UNCOMMITTED)
                        printf("SQL_TXN_READ_UNCOMMITTED\n");
                    else if (int_value == SQL_TXN_READ_COMMITTED)
                        printf("SQL_TXN_READ_COMMITTED\n");
                    else if (int_value == SQL_TXN_REPEATABLE_READ)
                        printf("SQL_TXN_REPEATABLE_READ\n");
                    else if (int_value == SQL_TXN_SERIALIZABLE)
                        printf("SQL_TXN_SERIALIZABLE\n");
                    else if (int_value == 0)
                        printf("Not supported\n");
                    else
                        printf("%u (Unknown)\n", int_value);
                } else {
                    printf("%u\n", int_value);
                }
            } else {
                printf("Error retrieving information\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
            break;
            
        case SQL_CURSOR_COMMIT_BEHAVIOR:
        case SQL_CURSOR_ROLLBACK_BEHAVIOR:
        case SQL_MAX_COLUMN_NAME_LEN:
        case SQL_MAX_CURSOR_NAME_LEN:
        case SQL_MAX_SCHEMA_NAME_LEN:
        case SQL_MAX_TABLE_NAME_LEN:
        case SQL_MAX_USER_NAME_LEN:
        case SQL_SQL_CONFORMANCE:
            ret = SQLGetInfo(handles->dbc_handle, info_type, &small_int_value, sizeof(small_int_value), NULL);
            if (is_success(ret)) {
                if (info_type == SQL_CURSOR_COMMIT_BEHAVIOR || info_type == SQL_CURSOR_ROLLBACK_BEHAVIOR) {
                    if (small_int_value == SQL_CB_DELETE)
                        printf("SQL_CB_DELETE\n");
                    else if (small_int_value == SQL_CB_CLOSE)
                        printf("SQL_CB_CLOSE\n");
                    else if (small_int_value == SQL_CB_PRESERVE)
                        printf("SQL_CB_PRESERVE\n");
                    else
                        printf("%u (Unknown)\n", small_int_value);
                } else if (info_type == SQL_SQL_CONFORMANCE) {
                    if (small_int_value == SQL_SC_SQL92_ENTRY)
                        printf("SQL_SC_SQL92_ENTRY\n");
                    else if (small_int_value == SQL_SC_FIPS127_2_TRANSITIONAL)
                        printf("SQL_SC_FIPS127_2_TRANSITIONAL\n");
                    else if (small_int_value == SQL_SC_SQL92_INTERMEDIATE)
                        printf("SQL_SC_SQL92_INTERMEDIATE\n");
                    else if (small_int_value == SQL_SC_SQL92_FULL)
                        printf("SQL_SC_SQL92_FULL\n");
                    else
                        printf("%u (Unknown)\n", small_int_value);
                } else {
                    printf("%u\n", small_int_value);
                }
            } else {
                printf("Error retrieving information\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
            break;
            
        case SQL_ACCESSIBLE_TABLES:
        case SQL_ACCESSIBLE_PROCEDURES:
        case SQL_DATA_SOURCE_READ_ONLY:
        case SQL_EXPRESSIONS_IN_ORDERBY:
        case SQL_MULT_RESULT_SETS:
        case SQL_PROCEDURES:
            ret = SQLGetInfo(handles->dbc_handle, info_type, buffer, sizeof(buffer), &buffer_len);
            if (is_success(ret)) {
                printf("%s (%s)\n", buffer, buffer[0] == 'Y' ? "Yes" : "No");
            } else {
                printf("Error retrieving information\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
            break;
            
        default: 
            printf("Unknown info type\n");
            break;
    }
}

void print_connection_attributes(OdbcHandles *handles) {
    SQLINTEGER attr_value;
    SQLINTEGER string_length;
    SQLRETURN ret;
    
    struct {
        SQLINTEGER attr;
        const char *name;
    } conn_attrs[] = {
        { SQL_ATTR_ACCESS_MODE, "Access Mode" },
        { SQL_ATTR_ASYNC_ENABLE, "Async Enable" },
        { SQL_ATTR_AUTO_IPD, "Auto IPD" },
        { SQL_ATTR_AUTOCOMMIT, "Autocommit" },
        { SQL_ATTR_CONNECTION_DEAD, "Connection Dead" },
        { SQL_ATTR_CONNECTION_TIMEOUT, "Connection Timeout" },
        { SQL_ATTR_CURRENT_CATALOG, "Current Catalog" },
        { SQL_ATTR_LOGIN_TIMEOUT, "Login Timeout" },
        { SQL_ATTR_METADATA_ID, "Metadata ID" },
        { SQL_ATTR_ODBC_CURSORS, "ODBC Cursors" },
        { SQL_ATTR_PACKET_SIZE, "Packet Size" },
        { SQL_ATTR_QUIET_MODE, "Quiet Mode" },
        { SQL_ATTR_TRACE, "Trace" },
        { SQL_ATTR_TRACEFILE, "Trace File" },
        { SQL_ATTR_TRANSLATE_LIB, "Translate Library" },
        { SQL_ATTR_TRANSLATE_OPTION, "Translate Option" },
        { SQL_ATTR_TXN_ISOLATION, "Transaction Isolation" },
        { 0, NULL }
    };
    
    for (int i = 0; conn_attrs[i].name != NULL; i++) {
        printf("%-30s: ", conn_attrs[i].name);
        
        if (conn_attrs[i].attr == SQL_ATTR_CURRENT_CATALOG ||
            conn_attrs[i].attr == SQL_ATTR_TRACEFILE ||
            conn_attrs[i].attr == SQL_ATTR_TRANSLATE_LIB) {
            
            SQLCHAR buffer[512];
            ret = SQLGetConnectAttr(handles->dbc_handle, conn_attrs[i].attr, 
                                  buffer, sizeof(buffer), &string_length);
            
            if (is_success(ret)) {
                printf("%s\n", buffer);
            } else {
                printf("Error retrieving attribute\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
        } else {
            ret = SQLGetConnectAttr(handles->dbc_handle, conn_attrs[i].attr, 
                                  &attr_value, sizeof(attr_value), &string_length);
            
            if (is_success(ret)) {
                if (conn_attrs[i].attr == SQL_ATTR_AUTOCOMMIT) {
                    printf("%s\n", attr_value == SQL_AUTOCOMMIT_ON ? "SQL_AUTOCOMMIT_ON" : "SQL_AUTOCOMMIT_OFF");
                } else if (conn_attrs[i].attr == SQL_ATTR_TXN_ISOLATION) {
                    if (attr_value == SQL_TXN_READ_UNCOMMITTED)
                        printf("SQL_TXN_READ_UNCOMMITTED\n");
                    else if (attr_value == SQL_TXN_READ_COMMITTED)
                        printf("SQL_TXN_READ_COMMITTED\n");
                    else if (attr_value == SQL_TXN_REPEATABLE_READ)
                        printf("SQL_TXN_REPEATABLE_READ\n");
                    else if (attr_value == SQL_TXN_SERIALIZABLE)
                        printf("SQL_TXN_SERIALIZABLE\n");
                    else if (attr_value == 0)
                        printf("Not supported\n");
                    else
                        printf("%ld (Unknown)\n", (long)attr_value);
                } else if (conn_attrs[i].attr == SQL_ATTR_ACCESS_MODE) {
                    printf("%s\n", attr_value == SQL_MODE_READ_ONLY ? "SQL_MODE_READ_ONLY" : 
                                  (attr_value == SQL_MODE_READ_WRITE ? "SQL_MODE_READ_WRITE" : 
                                   "Unknown"));
                } else if (conn_attrs[i].attr == SQL_ATTR_ASYNC_ENABLE) {
                    printf("%s\n", attr_value == SQL_ASYNC_ENABLE_ON ? "SQL_ASYNC_ENABLE_ON" : 
                                  (attr_value == SQL_ASYNC_ENABLE_OFF ? "SQL_ASYNC_ENABLE_OFF" : 
                                   "Unknown"));
                } else if (conn_attrs[i].attr == SQL_ATTR_CONNECTION_DEAD) {
                    printf("%s\n", attr_value == SQL_CD_TRUE ? "SQL_CD_TRUE (Dead)" : 
                                  (attr_value == SQL_CD_FALSE ? "SQL_CD_FALSE (Alive)" : 
                                   "Unknown"));
                } else {
                    printf("%ld\n", (long)attr_value);
                }
            } else {
                printf("Error retrieving attribute\n");
                print_odbc_errors(SQL_HANDLE_DBC, handles->dbc_handle);
            }
        }
    }
}
