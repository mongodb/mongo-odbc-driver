#include "common.h"

void test_error_scenarios(OdbcHandles *handles);
void print_error_details(SQLSMALLINT handle_type, SQLHANDLE handle);

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
    
    printf("\n--- MongoDB ODBC Error Analyzer ---\n\n");
    printf("This program demonstrates how to handle MongoDB-specific errors through ODBC.\n");
    
    OdbcHandles handles = init_odbc(SQL_OV_ODBC3);
    if (handles.env_handle == SQL_NULL_HANDLE || handles.dbc_handle == SQL_NULL_HANDLE) {
        cleanup_odbc(&handles);
        return 1;
    }
    
    if (!is_success(connect_to_mongodb(&handles, conn_str))) {
        printf("\nConnection error analysis:\n");
        print_error_details(SQL_HANDLE_DBC, handles.dbc_handle);
        cleanup_odbc(&handles);
        return 1;
    }
    
    handles.last_result = SQLAllocHandle(SQL_HANDLE_STMT, handles.dbc_handle, &handles.stmt_handle);
    if (!is_success(handles.last_result)) {
        fprintf(stderr, "Failed to allocate statement handle.\n");
        print_odbc_errors(SQL_HANDLE_DBC, handles.dbc_handle);
        cleanup_odbc(&handles);
        return 1;
    }
    
    test_error_scenarios(&handles);
    
    cleanup_odbc(&handles);
    return 0;
}

void test_error_scenarios(OdbcHandles *handles) {
    printf("\n1. Testing invalid SQL syntax:\n");
    handles->last_result = SQLExecDirect(handles->stmt_handle, 
                                       (SQLCHAR*)"SELECT * FROMM invalid_collection", SQL_NTS);
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
    
    printf("\n2. Testing non-existent collection:\n");
    handles->last_result = SQLExecDirect(handles->stmt_handle, 
                                       (SQLCHAR*)"SELECT * FROM non_existent_collection", SQL_NTS);
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
    
    printf("\n3. Testing invalid column reference:\n");
    handles->last_result = SQLExecDirect(handles->stmt_handle, 
                                       (SQLCHAR*)"SELECT non_existent_field FROM system.version", SQL_NTS);
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
    
    printf("\n4. Testing unsupported function:\n");
    handles->last_result = SQLExecDirect(handles->stmt_handle, 
                                       (SQLCHAR*)"SELECT UNSUPPORTED_FUNCTION() FROM system.version", SQL_NTS);
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
    
    printf("\n5. Testing invalid data type conversion:\n");
    handles->last_result = SQLExecDirect(handles->stmt_handle, 
                                       (SQLCHAR*)"SELECT CAST('invalid_date' AS DATE) FROM system.version", SQL_NTS);
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
    
    printf("\n6. Testing transaction support (not supported in MongoDB ODBC):\n");
    handles->last_result = SQLEndTran(SQL_HANDLE_DBC, handles->dbc_handle, SQL_COMMIT);
    printf("SQLEndTran result: %s\n", 
           is_success(handles->last_result) ? "Success (no-op)" : "Failed");
    if (!is_success(handles->last_result)) {
        print_error_details(SQL_HANDLE_DBC, handles->dbc_handle);
    }
}

void print_error_details(SQLSMALLINT handle_type, SQLHANDLE handle) {
    SQLSMALLINT i = 0;
    SQLINTEGER native;
    SQLCHAR state[7];
    SQLCHAR message[SQL_MAX_MESSAGE_LENGTH + 1];
    SQLSMALLINT len;
    SQLRETURN ret;
    
    printf("Error details:\n");
    
    do {
        ret = SQLGetDiagRec(handle_type, handle, ++i, state, &native, 
                          message, sizeof(message), &len);
        if (SQL_SUCCEEDED(ret)) {
            printf("  Record %d:\n", i);
            printf("    SQLSTATE: %s\n", state);
            printf("    Native Error: %d\n", (int)native);
            printf("    Message: %s\n", message);
            
            printf("    SQLSTATE Analysis: ");
            if (strncmp((char*)state, "01", 2) == 0) {
                printf("Warning\n");
            } else if (strncmp((char*)state, "07", 2) == 0) {
                printf("Dynamic SQL Error\n");
            } else if (strncmp((char*)state, "08", 2) == 0) {
                printf("Connection Error\n");
            } else if (strncmp((char*)state, "22", 2) == 0) {
                printf("Data Exception\n");
            } else if (strncmp((char*)state, "23", 2) == 0) {
                printf("Constraint Violation\n");
            } else if (strncmp((char*)state, "24", 2) == 0) {
                printf("Invalid Cursor State\n");
            } else if (strncmp((char*)state, "25", 2) == 0) {
                printf("Invalid Transaction State\n");
            } else if (strncmp((char*)state, "28", 2) == 0) {
                printf("Invalid Authorization\n");
            } else if (strncmp((char*)state, "42", 2) == 0) {
                printf("Syntax Error or Access Violation\n");
            } else if (strncmp((char*)state, "HY", 2) == 0) {
                printf("General Error\n");
            } else if (strncmp((char*)state, "IM", 2) == 0) {
                printf("Driver Manager Error\n");
            } else {
                printf("Other Error\n");
            }
            
            if (native != 0) {
                printf("    MongoDB Error Code: %d\n", (int)native);
                if (native >= 9001 && native <= 9999) {
                    printf("    MongoDB Category: Atlas Data Federation Error\n");
                } else if (native >= 8000 && native <= 8999) {
                    printf("    MongoDB Category: Shard Distribution Error\n");
                } else if (native >= 6000 && native <= 6999) {
                    printf("    MongoDB Category: Replication Error\n");
                } else if (native >= 5000 && native <= 5999) {
                    printf("    MongoDB Category: Sharding Error\n");
                } else if (native >= 4000 && native <= 4999) {
                    printf("    MongoDB Category: Network Error\n");
                } else if (native >= 3000 && native <= 3999) {
                    printf("    MongoDB Category: Storage Error\n");
                } else if (native >= 2000 && native <= 2999) {
                    printf("    MongoDB Category: Processing Error\n");
                } else if (native >= 1000 && native <= 1999) {
                    printf("    MongoDB Category: User Error\n");
                } else if (native >= 0 && native <= 999) {
                    printf("    MongoDB Category: Internal Error\n");
                }
            }
        }
    } while (SQL_SUCCEEDED(ret));
}
