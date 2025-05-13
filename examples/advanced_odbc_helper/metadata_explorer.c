#include "common.h"

void print_tables_result(SQLHSTMT stmt_handle);
void print_columns_result(SQLHSTMT stmt_handle);

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
    
    printf("\n--- MongoDB Metadata Explorer ---\n\n");
    
    printf("Listing all MongoDB databases (catalogs):\n");
    if (is_success(get_tables_metadata(&handles, "%", "", "", ""))) {
        print_tables_result(handles.stmt_handle);
    }
    
    printf("\nListing all MongoDB table types:\n");
    if (is_success(get_tables_metadata(&handles, "", "", "", "%"))) {
        print_tables_result(handles.stmt_handle);
    }
    
    printf("\nEnter a database name to list its collections (or press Enter for all): ");
    char database[256] = {0};
    fgets(database, sizeof(database), stdin);
    database[strcspn(database, "\r\n")] = 0;  // Remove newline
    
    const char *db_filter = (database[0] != '\0') ? database : "%";
    printf("\nListing collections for database(s) matching: %s\n", db_filter);
    
    if (is_success(get_tables_metadata(&handles, db_filter, "", "", "TABLE"))) {
        print_tables_result(handles.stmt_handle);
    }
    
    printf("\nEnter a database name to explore (or press Enter to skip): ");
    fgets(database, sizeof(database), stdin);
    database[strcspn(database, "\r\n")] = 0;  // Remove newline
    
    if (database[0] != '\0') {
        printf("Enter a collection name: ");
        char collection[256] = {0};
        fgets(collection, sizeof(collection), stdin);
        collection[strcspn(collection, "\r\n")] = 0;  // Remove newline
        
        if (collection[0] != '\0') {
            printf("\nListing columns for %s.%s:\n", database, collection);
            if (is_success(get_columns_metadata(&handles, database, "", collection, "%"))) {
                print_columns_result(handles.stmt_handle);
            }
        }
    }
    
    cleanup_odbc(&handles);
    return 0;
}

void print_tables_result(SQLHSTMT stmt_handle) {
    SQLRETURN ret;
    SQLCHAR catalog[256], schema[256], name[256], type[256], remarks[1024];
    SQLLEN catalog_len, schema_len, name_len, type_len, remarks_len;
    
    printf("%-20s %-20s %-30s %-15s %s\n", 
           "Catalog (Database)", "Schema", "Name (Collection)", "Type", "Remarks");
    printf("%-20s %-20s %-30s %-15s %s\n",
           "--------------------", "--------------------", "------------------------------", 
           "---------------", "-------------------");
    
    while ((ret = SQLFetch(stmt_handle)) == SQL_SUCCESS) {
        SQLGetData(stmt_handle, 1, SQL_C_CHAR, catalog, sizeof(catalog), &catalog_len);
        SQLGetData(stmt_handle, 2, SQL_C_CHAR, schema, sizeof(schema), &schema_len);
        SQLGetData(stmt_handle, 3, SQL_C_CHAR, name, sizeof(name), &name_len);
        SQLGetData(stmt_handle, 4, SQL_C_CHAR, type, sizeof(type), &type_len);
        SQLGetData(stmt_handle, 5, SQL_C_CHAR, remarks, sizeof(remarks), &remarks_len);
        
        printf("%-20s %-20s %-30s %-15s %s\n",
               catalog_len == SQL_NULL_DATA ? "(null)" : (char*)catalog,
               schema_len == SQL_NULL_DATA ? "(null)" : (char*)schema,
               name_len == SQL_NULL_DATA ? "(null)" : (char*)name,
               type_len == SQL_NULL_DATA ? "(null)" : (char*)type,
               remarks_len == SQL_NULL_DATA ? "(null)" : (char*)remarks);
    }
    
    if (ret != SQL_NO_DATA) {
        print_odbc_errors(SQL_HANDLE_STMT, stmt_handle);
    }
    
    SQLFreeStmt(stmt_handle, SQL_CLOSE);
}

void print_columns_result(SQLHSTMT stmt_handle) {
    SQLRETURN ret;
    SQLCHAR column_name[256], type_name[256];
    SQLLEN column_name_len, type_name_len;
    SQLSMALLINT data_type, decimal_digits, nullable;
    SQLLEN data_type_len, decimal_digits_len, nullable_len;
    SQLINTEGER column_size;
    SQLLEN column_size_len;
    
    printf("%-30s %-20s %-15s %-15s %-10s %s\n",
           "Column Name", "Type Name", "Data Type", "Column Size", "Decimals", "Nullable");
    printf("%-30s %-20s %-15s %-15s %-10s %s\n",
           "------------------------------", "--------------------", "---------------",
           "---------------", "----------", "--------");
    
    while ((ret = SQLFetch(stmt_handle)) == SQL_SUCCESS) {
        SQLGetData(stmt_handle, 4, SQL_C_CHAR, column_name, sizeof(column_name), &column_name_len);
        SQLGetData(stmt_handle, 6, SQL_C_SSHORT, &data_type, 0, &data_type_len);
        SQLGetData(stmt_handle, 7, SQL_C_CHAR, type_name, sizeof(type_name), &type_name_len);
        SQLGetData(stmt_handle, 8, SQL_C_SLONG, &column_size, 0, &column_size_len);
        SQLGetData(stmt_handle, 9, SQL_C_SSHORT, &decimal_digits, 0, &decimal_digits_len);
        SQLGetData(stmt_handle, 11, SQL_C_SSHORT, &nullable, 0, &nullable_len);
        
        printf("%-30s %-20s %-15d %-15d %-10d %s\n",
               column_name_len == SQL_NULL_DATA ? "(null)" : (char*)column_name,
               type_name_len == SQL_NULL_DATA ? "(null)" : (char*)type_name,
               data_type_len == SQL_NULL_DATA ? 0 : data_type,
               column_size_len == SQL_NULL_DATA ? 0 : column_size,
               decimal_digits_len == SQL_NULL_DATA ? 0 : decimal_digits,
               nullable_len == SQL_NULL_DATA ? "Unknown" : 
                   (nullable == SQL_NULLABLE ? "Yes" : 
                    (nullable == SQL_NO_NULLS ? "No" : "Unknown")));
    }
    
    if (ret != SQL_NO_DATA) {
        print_odbc_errors(SQL_HANDLE_STMT, stmt_handle);
    }
    
    SQLFreeStmt(stmt_handle, SQL_CLOSE);
}
