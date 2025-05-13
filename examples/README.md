# MongoDB ODBC Driver Examples

This directory contains examples demonstrating how to use the MongoDB ODBC driver in different ways:

## Standard ODBC API Examples

The `advanced_odbc_helper` directory contains examples that use the standard ODBC API to interact with MongoDB:

- **metadata_explorer**: Demonstrates how to use `SQLTables` and `SQLColumns` to explore MongoDB databases and collections
- **error_analyzer**: Shows how to handle and analyze MongoDB-specific errors
- **connection_info_utility**: Retrieves and displays detailed information about the MongoDB ODBC connection
- **data_type_handler**: Demonstrates how to work with MongoDB-specific data types
- **mongodb_x509_connector**: Shows X.509 certificate authentication with flexible certificate path configuration

## Direct API Examples

The `direct_api` directory contains examples that use the MongoDB ODBC driver core API directly, bypassing the standard ODBC API:

- **direct_mongo_connector**: Demonstrates how to connect to MongoDB, execute queries, and fetch results using the direct API

## Hybrid API Examples

The `hybrid_api` directory contains examples that use both the standard ODBC API and the direct MongoDB ODBC driver core API:

- **hybrid_mongo_connector**: Demonstrates how to use both APIs side by side for comparison

## Building and Running the Examples

### Standard ODBC API Examples

```bash
cd advanced_odbc_helper
make
./metadata_explorer "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/"
```

### Direct API Examples

```bash
cd direct_api
make
./direct_mongo_connector "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/" "SELECT * FROM system.version"
```

### Hybrid API Examples

```bash
cd hybrid_api
make
./hybrid_mongo_connector "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/" "SELECT * FROM system.version"
```

## Prerequisites

- MongoDB ODBC Driver installed and configured
- UnixODBC development libraries
- GCC or compatible C compiler
```
