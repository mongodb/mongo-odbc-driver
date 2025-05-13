# MongoDB Advanced ODBC Helper

A collection of advanced C programs that demonstrate how to use the MongoDB ODBC driver with sophisticated error handling, metadata operations, and data type management. These examples follow the ODBC API call sequence as shown in real-world ODBC traces and provide comprehensive error handling.

## Prerequisites

- MongoDB ODBC Driver installed and configured
- UnixODBC development libraries
- GCC or compatible C compiler

## Building

```bash
make
```

## Examples

### 1. Metadata Explorer

Demonstrates how to use `SQLTables` and `SQLColumns` to explore MongoDB databases, collections, and fields.

```bash
./metadata_explorer "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/"
```

### 2. Error Analyzer

Shows how to handle and analyze MongoDB-specific errors through the ODBC interface.

```bash
./error_analyzer "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/"
```

### 3. Connection Info Utility

Retrieves and displays detailed information about the MongoDB ODBC connection.

```bash
./connection_info_utility "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/"
```

### 4. Data Type Handler

Demonstrates how to work with MongoDB-specific data types through ODBC.

```bash
./data_type_handler "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/" "collection_name"
```

### 5. MongoDB X.509 Connector

Demonstrates X.509 certificate authentication with enhanced error handling and flexible certificate path configuration.

```bash
./mongodb_x509_connector -c /path/to/client.pem -a /path/to/ca.pem -u mongodb://localhost:27017/
```

## ODBC Trace Sequence

These examples implement the following ODBC call sequence from the trace:

1. SQLAllocHandle (SQL_HANDLE_ENV)
2. SQLSetEnvAttr (SQL_ATTR_ODBC_VERSION)
3. SQLAllocHandle (SQL_HANDLE_DBC)
4. SQLSetConnectAttr (SQL_ATTR_LOGIN_TIMEOUT)
5. SQLDriverConnect
6. SQLAllocHandle (SQL_HANDLE_STMT)
7. SQLExecDirect / SQLTables / SQLColumns
8. SQLFetch
9. SQLGetData
10. SQLMoreResults
11. SQLFreeHandle (SQL_HANDLE_STMT)
12. SQLDisconnect
13. SQLFreeHandle (SQL_HANDLE_DBC)
14. SQLFreeHandle (SQL_HANDLE_ENV)

## Error Handling

The examples include comprehensive error handling that:

1. Checks return codes from all ODBC API calls
2. Retrieves and displays detailed error information using SQLGetDiagRec
3. Properly cleans up resources in error scenarios
4. Provides meaningful error messages to help diagnose connection issues
5. Handles MongoDB-specific error codes and messages

## MongoDB-Specific Features

These examples demonstrate MongoDB-specific features:

1. Working with MongoDB data types like ObjectId, ISODate, etc.
2. X.509 certificate authentication with flexible certificate path configuration
3. Metadata operations for MongoDB databases and collections
4. Handling MongoDB-specific error codes and messages
5. Retrieving MongoDB server information

## Implementation Details

The examples use a common utility library (`common.c` and `common.h`) that provides:

1. ODBC handle management
2. Error handling and reporting
3. Connection management
4. Query execution
5. Metadata retrieval
6. Data type handling

This modular approach makes the examples easier to understand and maintain.

## Troubleshooting

### Common Issues

1. **Driver Not Found**: Ensure the MongoDB ODBC Driver is properly installed and configured in your odbcinst.ini file.

2. **Connection Failures**: Check that the MongoDB server is running and accessible. Verify that the connection string is correct.

3. **Authentication Errors**: For X.509 authentication, ensure that certificate paths are correct and the certificates are valid.

4. **Data Type Errors**: MongoDB has specific data types that may not map directly to SQL types. The data_type_handler example shows how to handle these conversions.

### Debugging Tips

1. Enable ODBC tracing in your odbcinst.ini file to capture detailed ODBC API calls.

2. Use the error_analyzer example to understand specific error codes and messages.

3. Check MongoDB server logs for additional error information.

4. Use the connection_info_utility to verify driver and connection settings.
