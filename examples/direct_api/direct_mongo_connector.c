/**
 * @file direct_mongo_connector.c
 * @brief Example demonstrating direct use of the MongoDB ODBC driver core API
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "mongo_odbc_ffi.h"

void print_error(MongoOdbcErrorCode error_code) {
    const char* message = mongo_odbc_get_error_message(error_code);
    printf("Error: %s (code %d)\n", message, error_code);
}

int main(int argc, char** argv) {
    const char* connection_string = "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/";
    const char* query = "SELECT * FROM system.version";
    MongoOdbcErrorCode error_code = MONGO_ODBC_SUCCESS;
    
    if (argc > 1) {
        connection_string = argv[1];
    }
    if (argc > 2) {
        query = argv[2];
    }
    
    printf("Connection string: %s\n", connection_string);
    printf("Query: %s\n", query);
    
    printf("Connecting to MongoDB...\n");
    ConnectionHandle* connection = mongo_odbc_connect(connection_string, &error_code);
    if (!connection) {
        print_error(error_code);
        return 1;
    }
    printf("Connected successfully\n");
    
    printf("Preparing query...\n");
    StatementHandle* statement = mongo_odbc_prepare_query(connection, query, &error_code);
    if (!statement) {
        print_error(error_code);
        mongo_odbc_free_connection(connection);
        return 1;
    }
    printf("Query prepared successfully\n");
    
    printf("Executing query...\n");
    bool success = mongo_odbc_execute_statement(connection, statement, &error_code);
    if (!success) {
        print_error(error_code);
        mongo_odbc_free_statement(statement);
        mongo_odbc_free_connection(connection);
        return 1;
    }
    printf("Query executed successfully\n");
    
    printf("\nResults:\n");
    int row_count = 0;
    while (mongo_odbc_fetch(statement, &error_code)) {
        row_count++;
        printf("Row %d fetched\n", row_count);
        
    }
    
    if (error_code != MONGO_ODBC_SUCCESS) {
        print_error(error_code);
    }
    
    printf("\nTotal rows: %d\n", row_count);
    
    mongo_odbc_free_statement(statement);
    mongo_odbc_free_connection(connection);
    
    printf("Connection closed\n");
    
    return 0;
}
