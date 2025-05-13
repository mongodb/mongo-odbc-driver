#include "common.h"
#include <unistd.h>
#include <getopt.h>

void show_usage(const char *program_name) {
    printf("Usage: %s [options]\n", program_name);
    printf("Options:\n");
    printf("  -h, --help                 Show this help message\n");
    printf("  -u, --uri <uri>            MongoDB URI (default: mongodb://localhost:27017/)\n");
    printf("  -d, --driver <driver>      ODBC Driver name (default: MongoDB ODBC Driver)\n");
    printf("  -c, --client-cert <path>   Path to client certificate PEM file\n");
    printf("  -a, --ca-cert <path>       Path to CA certificate PEM file\n");
    printf("  -q, --query <query>        SQL query to execute (default: SELECT * FROM system.version)\n");
    printf("  -v, --verbose              Enable verbose output\n\n");
    printf("Example: %s -c /path/to/client.pem -a /path/to/ca.pem\n", program_name);
}

int main(int argc, char **argv) {
    char *uri = NULL;
    char *driver = NULL;
    char *client_cert = NULL;
    char *ca_cert = NULL;
    char *query = NULL;
    int verbose = 0;
    
    static struct option long_options[] = {
        {"help",        no_argument,       0, 'h'},
        {"uri",         required_argument, 0, 'u'},
        {"driver",      required_argument, 0, 'd'},
        {"client-cert", required_argument, 0, 'c'},
        {"ca-cert",     required_argument, 0, 'a'},
        {"query",       required_argument, 0, 'q'},
        {"verbose",     no_argument,       0, 'v'},
        {0, 0, 0, 0}
    };
    
    int option_index = 0;
    int c;
    
    while ((c = getopt_long(argc, argv, "hu:d:c:a:q:v", long_options, &option_index)) != -1) {
        switch (c) {
            case 'h':
                show_usage(argv[0]);
                return 0;
            case 'u':
                uri = optarg;
                break;
            case 'd':
                driver = optarg;
                break;
            case 'c':
                client_cert = optarg;
                break;
            case 'a':
                ca_cert = optarg;
                break;
            case 'q':
                query = optarg;
                break;
            case 'v':
                verbose = 1;
                break;
            case '?':
                return 1;
            default:
                abort();
        }
    }
    
    if (!uri) uri = "mongodb://localhost:27017/";
    if (!driver) driver = "MongoDB ODBC Driver";
    if (!query) query = "SELECT * FROM system.version";
    
    if ((client_cert && !ca_cert) || (!client_cert && ca_cert)) {
        fprintf(stderr, "Error: Both client certificate and CA certificate must be provided for X.509 authentication.\n");
        return 1;
    }
    
    if (client_cert && access(client_cert, R_OK) != 0) {
        fprintf(stderr, "Error: Cannot access client certificate file: %s\n", client_cert);
        return 1;
    }
    
    if (ca_cert && access(ca_cert, R_OK) != 0) {
        fprintf(stderr, "Error: Cannot access CA certificate file: %s\n", ca_cert);
        return 1;
    }
    
    char conn_str[2048] = {0};
    
    if (client_cert && ca_cert) {
        snprintf(conn_str, sizeof(conn_str),
                "Driver={%s};URI=%s?authSource=$external&authMechanism=MONGODB-X509;"
                "sslClientCertificateKeyFile=%s;sslCAFile=%s;",
                driver, uri, client_cert, ca_cert);
    } else {
        snprintf(conn_str, sizeof(conn_str), "Driver={%s};URI=%s;", driver, uri);
    }
    
    if (verbose) {
        printf("Connection string: %s\n", conn_str);
    }
    
    OdbcHandles handles = init_odbc(SQL_OV_ODBC3);
    if (handles.env_handle == SQL_NULL_HANDLE || handles.dbc_handle == SQL_NULL_HANDLE) {
        cleanup_odbc(&handles);
        return 1;
    }
    
    printf("Connecting to MongoDB using %s authentication...\n", 
           (client_cert && ca_cert) ? "X.509" : "standard");
    
    if (!is_success(connect_to_mongodb(&handles, conn_str))) {
        fprintf(stderr, "Connection failed.\n");
        cleanup_odbc(&handles);
        return 1;
    }
    
    if (verbose) {
        char dbms_name[256] = {0};
        char dbms_ver[256] = {0};
        SQLSMALLINT name_len, ver_len;
        
        printf("\nConnection Information:\n");
        
        if (is_success(get_connection_info(&handles, SQL_DBMS_NAME, dbms_name, sizeof(dbms_name), &name_len))) {
            printf("  DBMS Name: %s\n", dbms_name);
        }
        
        if (is_success(get_connection_info(&handles, SQL_DBMS_VER, dbms_ver, sizeof(dbms_ver), &ver_len))) {
            printf("  DBMS Version: %s\n", dbms_ver);
        }
        
        SQLUINTEGER timeout;
        SQLINTEGER timeout_len;
        
        if (is_success(get_connection_attr(&handles, SQL_ATTR_LOGIN_TIMEOUT, 
                                         &timeout, sizeof(timeout), &timeout_len))) {
            printf("  Login Timeout: %u seconds\n", timeout);
        }
        
        char driver_name[256] = {0};
        char driver_ver[256] = {0};
        SQLSMALLINT driver_name_len, driver_ver_len;
        
        if (is_success(get_connection_info(&handles, SQL_DRIVER_NAME, 
                                         driver_name, sizeof(driver_name), &driver_name_len))) {
            printf("  Driver Name: %s\n", driver_name);
        }
        
        if (is_success(get_connection_info(&handles, SQL_DRIVER_VER, 
                                         driver_ver, sizeof(driver_ver), &driver_ver_len))) {
            printf("  Driver Version: %s\n", driver_ver);
        }
    }
    
    printf("\nExecuting query: %s\n", query);
    
    if (is_success(execute_query(&handles, query))) {
        printf("Query executed successfully.\n");
        
        SQLRETURN ret;
        SQLSMALLINT col_count;
        
        ret = SQLNumResultCols(handles.stmt_handle, &col_count);
        if (!is_success(ret)) {
            fprintf(stderr, "Failed to get column count.\n");
            print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
            cleanup_odbc(&handles);
            return 1;
        }
        
        printf("\nResult set has %d column(s):\n", col_count);
        
        SQLCHAR col_name[256];
        SQLSMALLINT col_name_len, data_type, decimal_digits, nullable;
        SQLULEN col_size;
        
        for (SQLSMALLINT i = 1; i <= col_count; i++) {
            ret = SQLDescribeCol(handles.stmt_handle, i, col_name, sizeof(col_name), 
                               &col_name_len, &data_type, &col_size, &decimal_digits, &nullable);
            if (is_success(ret)) {
                printf("  Column %d: %s (SQL Type: %d)\n", i, col_name, data_type);
            }
        }
        
        printf("\nResults:\n");
        int row_count = 0;
        
        printf("| ");
        for (SQLSMALLINT i = 1; i <= col_count; i++) {
            ret = SQLDescribeCol(handles.stmt_handle, i, col_name, sizeof(col_name), 
                               &col_name_len, &data_type, &col_size, &decimal_digits, &nullable);
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
            row_count++;
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
        } else {
            printf("\nTotal rows: %d\n", row_count);
        }
        
        while (SQLMoreResults(handles.stmt_handle) == SQL_SUCCESS) {
            printf("\nAdditional result set found.\n");
            
            ret = SQLNumResultCols(handles.stmt_handle, &col_count);
            if (!is_success(ret)) {
                fprintf(stderr, "Failed to get column count for additional result set.\n");
                print_odbc_errors(SQL_HANDLE_STMT, handles.stmt_handle);
                break;
            }
            
            printf("Additional result set has %d column(s).\n", col_count);
            
            row_count = 0;
            while (SQLFetch(handles.stmt_handle) == SQL_SUCCESS) {
                row_count++;
            }
            printf("Additional result set rows: %d\n", row_count);
        }
    }
    
    cleanup_odbc(&handles);
    printf("\nConnection closed.\n");
    
    return 0;
}
