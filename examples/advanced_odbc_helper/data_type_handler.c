#include "common.h"

void print_type_info(SQLHSTMT stmt_handle);
void test_data_type_handling(OdbcHandles *handles, const char *collection);
void detect_mongodb_type(const char *value);

void show_usage(const char *program_name) {
    printf("Usage: %s [connection_string] [collection_name]\n", program_name);
    printf("  connection_string: ODBC connection string (optional)\n");
    printf("  collection_name: Collection to query (optional)\n\n");
    printf("Example: %s \"Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/\" test_collection\n", 
           program_name);
}

int main(int argc, char **argv) {
    const char *conn_str = (argc > 1) ? argv[1] : 
        "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/";
    const char *collection = (argc > 2) ? argv[2] : "system.version";
    
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
    
    printf("\n--- MongoDB ODBC Data Type Handler ---\n\n");
    
    handles.last_result = SQLAllocHandle(SQL_HANDLE_STMT, handles.dbc_handle, &handles.stmt_handle);
    if (!is_success(handles.last_result)) {
        fprintf(stderr, "Failed to allocate statement handle.\n");
        print_odbc_errors(SQL_HANDLE_DBC, handles.dbc_handle);
        cleanup_odbc(&handles);
        return 1;
    }
    
    printf("Supported SQL Data Types:\n");
    handles.last_result = SQLGetTypeInfo(handles.stmt_handle, SQL_ALL_TYPES);
    if (is_success(handles.last_result)) {
        print_type_info(handles.stmt_handle);
    } else {
        fprintf(stderr, "Failed to get type information.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
    }
    
    test_data_type_handling(&handles, collection);
    
    printf("\nTesting MongoDB-specific data types:\n");
    
    const char *create_test_query = 
        "CREATE TABLE IF NOT EXISTS odbc_test_types ("
        "  _id STRING, "
        "  objectid_field STRING, "
        "  string_field STRING, "
        "  int32_field INT, "
        "  int64_field BIGINT, "
        "  double_field DOUBLE, "
        "  bool_field BOOLEAN, "
        "  date_field TIMESTAMP, "
        "  null_field STRING, "
        "  array_field STRING, "
        "  object_field STRING, "
        "  binary_field STRING"
        ")";
    
    printf("Creating test table with MongoDB data types...\n");
    if (is_success(execute_query(&handles, create_test_query))) {
        printf("Test table created successfully.\n");
        
        const char *insert_query = 
            "INSERT INTO odbc_test_types VALUES ("
            "  'test_id', "
            "  '{\"$oid\":\"507f1f77bcf86cd799439011\"}', "
            "  'test string', "
            "  42, "
            "  9223372036854775807, "
            "  3.14159, "
            "  true, "
            "  '2023-05-13T12:34:56.789Z', "
            "  NULL, "
            "  '[1, 2, 3, \"four\", {\"five\": 5}]', "
            "  '{\"nested\": {\"field\": \"value\"}}', "
            "  '{\"$binary\":\"dGVzdCBiaW5hcnkgZGF0YQ==\", \"$type\":\"00\"}'"
            ")";
        
        printf("Inserting test data...\n");
        if (is_success(execute_query(&handles, insert_query))) {
            printf("Test data inserted successfully.\n");
            
            printf("\nQuerying test data with MongoDB types:\n");
            if (is_success(execute_query(&handles, "SELECT * FROM odbc_test_types"))) {
                SQLRETURN ret;
                SQLSMALLINT col_count;
                
                ret = SQLNumResultCols(handles.stmt_handle, &col_count);
                if (!is_success(ret)) {
                    fprintf(stderr, "Failed to get column count.\n");
                    print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
                } else {
                    printf("| ");
                    for (SQLSMALLINT i = 1; i <= col_count; i++) {
                        SQLCHAR col_name[256];
                        SQLSMALLINT col_name_len;
                        
                        ret = SQLColAttribute(handles.stmt_handle, i, SQL_DESC_NAME,
                                           col_name, sizeof(col_name), &col_name_len, NULL);
                        if (is_success(ret)) {
                            printf("%-20s | ", col_name);
                        } else {
                            printf("%-20s | ", "Column");
                        }
                    }
                    printf("\n");
                    
                    printf("|");
                    for (SQLSMALLINT i = 1; i <= col_count; i++) {
                        printf("----------------------|");
                    }
                    printf("\n");
                    
                    while ((ret = SQLFetch(handles.stmt_handle)) == SQL_SUCCESS) {
                        printf("| ");
                        
                        for (SQLSMALLINT i = 1; i <= col_count; i++) {
                            SQLCHAR buffer[1024];
                            SQLLEN indicator;
                            
                            ret = SQLGetData(handles.stmt_handle, i, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                            
                            if (is_success(ret)) {
                                if (indicator == SQL_NULL_DATA) {
                                    printf("%-20s | ", "NULL");
                                } else {
                                    if (strlen((char*)buffer) > 20) {
                                        buffer[17] = '.';
                                        buffer[18] = '.';
                                        buffer[19] = '.';
                                        buffer[20] = '\0';
                                    }
                                    printf("%-20s | ", buffer);
                                }
                            } else {
                                printf("%-20s | ", "[ERROR]");
                                print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
                            }
                        }
                        printf("\n");
                    }
                    
                    if (ret != SQL_NO_DATA) {
                        fprintf(stderr, "Error fetching data.\n");
                        print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
                    }
                }
            } else {
                fprintf(stderr, "Failed to query test data.\n");
                print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
            }
            
            printf("\nAnalyzing MongoDB-specific types:\n");
            if (is_success(execute_query(&handles, "SELECT * FROM odbc_test_types"))) {
                SQLRETURN ret;
                
                if ((ret = SQLFetch(handles.stmt_handle)) == SQL_SUCCESS) {
                    SQLCHAR buffer[1024];
                    SQLLEN indicator;
                    
                    ret = SQLGetData(handles.stmt_handle, 2, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                    if (is_success(ret) && indicator != SQL_NULL_DATA) {
                        printf("ObjectId field: %s\n", buffer);
                        detect_mongodb_type((char*)buffer);
                    }
                    
                    ret = SQLGetData(handles.stmt_handle, 8, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                    if (is_success(ret) && indicator != SQL_NULL_DATA) {
                        printf("Date field: %s\n", buffer);
                        detect_mongodb_type((char*)buffer);
                    }
                    
                    ret = SQLGetData(handles.stmt_handle, 10, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                    if (is_success(ret) && indicator != SQL_NULL_DATA) {
                        printf("Array field: %s\n", buffer);
                        detect_mongodb_type((char*)buffer);
                    }
                    
                    ret = SQLGetData(handles.stmt_handle, 11, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                    if (is_success(ret) && indicator != SQL_NULL_DATA) {
                        printf("Object field: %s\n", buffer);
                        detect_mongodb_type((char*)buffer);
                    }
                    
                    ret = SQLGetData(handles.stmt_handle, 12, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
                    if (is_success(ret) && indicator != SQL_NULL_DATA) {
                        printf("Binary field: %s\n", buffer);
                        detect_mongodb_type((char*)buffer);
                    }
                }
            }
            
            printf("\nCleaning up test table...\n");
            if (is_success(execute_query(&handles, "DROP TABLE odbc_test_types"))) {
                printf("Test table dropped successfully.\n");
            } else {
                fprintf(stderr, "Failed to drop test table.\n");
                print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
            }
        } else {
            fprintf(stderr, "Failed to insert test data.\n");
            print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
        }
    } else {
        fprintf(stderr, "Failed to create test table.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
    }
    
    cleanup_odbc(&handles);
    return 0;
}

void print_type_info(SQLHSTMT stmt_handle) {
    SQLRETURN ret;
    SQLCHAR type_name[256], literal_prefix[10], literal_suffix[10];
    SQLLEN type_name_len, prefix_len, suffix_len;
    SQLSMALLINT data_type, nullable;
    SQLLEN data_type_len, nullable_len;
    SQLULEN column_size;
    SQLLEN column_size_len;
    
    printf("%-25s %-15s %-15s %-10s %-10s %s\n",
           "Type Name", "SQL Type", "Column Size", "Prefix", "Suffix", "Nullable");
    printf("%-25s %-15s %-15s %-10s %-10s %s\n",
           "-------------------------", "---------------", "---------------",
           "----------", "----------", "--------");
    
    while ((ret = SQLFetch(stmt_handle)) == SQL_SUCCESS) {
        SQLGetData(stmt_handle, 1, SQL_C_CHAR, type_name, sizeof(type_name), &type_name_len);
        SQLGetData(stmt_handle, 2, SQL_C_SSHORT, &data_type, 0, &data_type_len);
        SQLGetData(stmt_handle, 3, SQL_C_ULONG, &column_size, 0, &column_size_len);
        SQLGetData(stmt_handle, 4, SQL_C_CHAR, literal_prefix, sizeof(literal_prefix), &prefix_len);
        SQLGetData(stmt_handle, 5, SQL_C_CHAR, literal_suffix, sizeof(literal_suffix), &suffix_len);
        SQLGetData(stmt_handle, 7, SQL_C_SSHORT, &nullable, 0, &nullable_len);
        
        printf("%-25s %-15d %-15lu %-10s %-10s %s\n",
               type_name_len == SQL_NULL_DATA ? "(null)" : (char*)type_name,
               data_type_len == SQL_NULL_DATA ? 0 : data_type,
               column_size_len == SQL_NULL_DATA ? 0 : column_size,
               prefix_len == SQL_NULL_DATA ? "(null)" : (char*)literal_prefix,
               suffix_len == SQL_NULL_DATA ? "(null)" : (char*)literal_suffix,
               nullable_len == SQL_NULL_DATA ? "Unknown" : 
                   (nullable == SQL_NULLABLE ? "Yes" : 
                    (nullable == SQL_NO_NULLS ? "No" : "Unknown")));
    }
    
    if (ret != SQL_NO_DATA) {
        fprintf(stderr, "Error fetching type information.\n");
    }
    
    SQLFreeStmt(stmt_handle, SQL_CLOSE);
}

void test_data_type_handling(OdbcHandles *handles, const char *collection) {
    printf("\nTesting data type handling with collection: %s\n", collection);
    
    char query[512];
    snprintf(query, sizeof(query), "SELECT * FROM %s LIMIT 1", collection);
    
    handles->last_result = SQLExecDirect(handles->stmt_handle, (SQLCHAR*)query, SQL_NTS);
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to execute query: %s\n", query);
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
        return;
    }
    
    SQLSMALLINT col_count;
    handles->last_result = SQLNumResultCols(handles->stmt_handle, &col_count);
    if (!is_success(handles->last_result)) {
        fprintf(stderr, "Failed to get column count.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
        return;
    }
    
    printf("\nColumn metadata for %s:\n", collection);
    printf("%-20s %-15s %-15s %-10s %-10s\n",
           "Column Name", "SQL Type", "Column Size", "Decimals", "Nullable");
    printf("%-20s %-15s %-15s %-10s %-10s\n",
           "--------------------", "---------------", "---------------",
           "----------", "----------");
    
    for (SQLSMALLINT i = 1; i <= col_count; i++) {
        SQLCHAR col_name[256];
        SQLSMALLINT col_name_len, data_type, decimal_digits, nullable;
        SQLULEN col_size;
        
        handles->last_result = SQLDescribeCol(handles->stmt_handle, i, col_name, sizeof(col_name),
                                            &col_name_len, &data_type, &col_size, 
                                            &decimal_digits, &nullable);
        
        if (is_success(handles->last_result)) {
            printf("%-20s %-15d %-15lu %-10d %-10s\n",
                   col_name,
                   data_type,
                   col_size,
                   decimal_digits,
                   nullable == SQL_NULLABLE ? "Yes" : 
                       (nullable == SQL_NO_NULLS ? "No" : "Unknown"));
        } else {
            fprintf(stderr, "Failed to get column information for column %d.\n", i);
            print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
        }
    }
    
    printf("\nData with type information:\n");
    
    if (SQLFetch(handles->stmt_handle) == SQL_SUCCESS) {
        for (SQLSMALLINT i = 1; i <= col_count; i++) {
            SQLCHAR col_name[256];
            SQLSMALLINT col_name_len, data_type, decimal_digits, nullable;
            SQLULEN col_size;
            
            SQLDescribeCol(handles->stmt_handle, i, col_name, sizeof(col_name),
                         &col_name_len, &data_type, &col_size, &decimal_digits, &nullable);
            
            SQLCHAR buffer[8192];  // Large buffer for any data type
            SQLLEN indicator;
            
            handles->last_result = SQLGetData(handles->stmt_handle, i, SQL_C_CHAR, 
                                            buffer, sizeof(buffer), &indicator);
            
            if (is_success(handles->last_result)) {
                printf("Column %d (%s):\n", i, col_name);
                printf("  SQL Type: %d\n", data_type);
                
                if (indicator == SQL_NULL_DATA) {
                    printf("  Value: NULL\n");
                } else {
                    printf("  Value: %s\n", buffer);
                    
                    switch (data_type) {
                        case SQL_CHAR:
                        case SQL_VARCHAR:
                        case SQL_LONGVARCHAR:
                        case SQL_WCHAR:
                        case SQL_WVARCHAR:
                        case SQL_WLONGVARCHAR:
                            printf("  Type: String\n");
                            printf("  Length: %ld\n", (long)strlen((char*)buffer));
                            break;
                            
                        case SQL_DECIMAL:
                        case SQL_NUMERIC:
                        case SQL_REAL:
                        case SQL_FLOAT:
                        case SQL_DOUBLE:
                            printf("  Type: Numeric\n");
                            break;
                            
                        case SQL_INTEGER:
                        case SQL_SMALLINT:
                        case SQL_TINYINT:
                        case SQL_BIGINT:
                            printf("  Type: Integer\n");
                            break;
                            
                        case SQL_TYPE_DATE:
                        case SQL_TYPE_TIME:
                        case SQL_TYPE_TIMESTAMP:
                            printf("  Type: Date/Time\n");
                            break;
                            
                        case SQL_BINARY:
                        case SQL_VARBINARY:
                        case SQL_LONGVARBINARY:
                            printf("  Type: Binary\n");
                            break;
                            
                        case SQL_BIT:
                            printf("  Type: Boolean\n");
                            break;
                            
                        default:
                            printf("  Type: Other\n");
                            break;
                    }
                    
                    detect_mongodb_type((char*)buffer);
                }
            } else {
                fprintf(stderr, "Failed to get data for column %d.\n", i);
                print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
            }
            
            printf("\n");
        }
    } else {
        fprintf(stderr, "Failed to fetch data.\n");
        print_odbc_errors(SQL_HANDLE_STMT, handles->stmt_handle);
    }
    
    SQLFreeStmt(handles->stmt_handle, SQL_CLOSE);
}

void detect_mongodb_type(const char *value) {
    if (!value) return;
    
    if (strstr(value, "{\"$oid\":") != NULL) {
        printf("  MongoDB Type: ObjectId\n");
    } else if (strstr(value, "ISODate(") != NULL || 
               strstr(value, "{\"$date\":") != NULL) {
        printf("  MongoDB Type: ISODate\n");
    } else if (strstr(value, "NumberLong(") != NULL || 
               strstr(value, "{\"$numberLong\":") != NULL) {
        printf("  MongoDB Type: NumberLong\n");
    } else if (strstr(value, "NumberDecimal(") != NULL || 
               strstr(value, "{\"$numberDecimal\":") != NULL) {
        printf("  MongoDB Type: NumberDecimal\n");
    } else if (strstr(value, "BinData(") != NULL || 
               strstr(value, "{\"$binary\":") != NULL) {
        printf("  MongoDB Type: BinData\n");
    } else if (strstr(value, "{\"$timestamp\":") != NULL) {
        printf("  MongoDB Type: Timestamp\n");
    } else if (strstr(value, "{\"$regex\":") != NULL) {
        printf("  MongoDB Type: Regex\n");
    } else if (value[0] == '[' && value[strlen(value) - 1] == ']') {
        printf("  MongoDB Type: Array\n");
    } else if (value[0] == '{' && value[strlen(value) - 1] == '}') {
        printf("  MongoDB Type: Document/Object\n");
    }
}
