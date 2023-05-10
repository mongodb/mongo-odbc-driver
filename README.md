# mongo-odbc-driver

An 64-bit unicode ODBC driver for connecting to Atlas Data Lake via the MongoSQL dialect.

If you're looking for an ODBC driver to use with the [MongoDB
Connector for BI](https://docs.mongodb.com/bi-connector/current/),
please see the
[mongodb/mongo-bi-connector-odbc-driver](https://github.com/mongodb/mongo-bi-connector-odbc-driver)
repository.


## Set up the ODBC driver on Windows
Note: users can utilize the built-in driver manager.
1. Update the values of `Driver`, `Pwd`, `Server`, `User`, and `Database` in `setup/setupDSN.reg`. The value of `Driver` should be the absolute path of `mongoodbc.dll`. This file should be located in either the `mongo-odbc-driver/target/debug` directory or in the release directory.

2. For 32-bit architectures, modify the file path `HKEY_LOCAL_MACHINE\SOFTWARE` in `setupDSN.reg` so that it is instead `HKEY_LOCAL_MACHINE\Wow6432Node\SOFTWARE`.

3. Run `reg import "setup/setupDSN.reg"` in order to populate the registry editor with the new entries. Alternatively, simply double click on the `setupDSN.reg` file.

## Validate setup
### 64-bit
Run `reg query "HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers"` to verify that `MongoDB Atlas SQL ODBC Driver` has been installed successfully.

There should be a new entry called `ADF_TEST` under `ODBC/ODBC.INI` with the following subentries:

	
	database: <database name>
	pwd: <password>
	server: <server>
	user: <user>
	

Run `reg query "HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI\MongoDB Atlas SQL ODBC Driver"` to determine if the registry editor was updated successfully. There should also be a new entry called `ADF_ODBC` under `ODBC/ODBCINST.INI` with the following subentries:

	
    Driver: <path to dll>
    Setup: <path to dll>
	


Open the Microsoft ODBC Administrator (64-bit) and verify that "MongoDB Atlas SQL ODBC Driver" appears under "System DSN".

## Unsupported Functions

The driver is a Unicode only driver and does not support ANSI functions.  
Additionally, the following ODBC functions are currently not supported by the driver. 

| Function             |
|----------------------|
| SQLBindCol           |
| SQLBindParameter     |
| SQLBrowseConnectW    |
| SQLBulkOperations    |
| SQLCancel            |
| SQLCancelHandle      |
| SQLColumnPrivilegesW |
| SQLCompleteAsync     |
| SQLConnectW          |
| SQLCopyDesc          |
| SQLDescribeParam     |
| SQLEndTran           |
| SQLFetchScroll       |
| SQLFreeStmt          |
| SQLGetCursorNameW    |
| SQLGetDescFieldW     |
| SQLGetDescRecW       |
| SQLNativeSqlW        |
| SQLNumParams         |
| SQLParamData         |
| SQLPrepareW          |
| SQLProcedureColumnsW |
| SQLProceduresW       |
| SQLPutData           |
| SQLSetCursorNameW    |
| SQLSetDescFieldW     |
| SQLSetDescRec        |
| SQLSetPos            |
| SQLSpecialColumnsW   |
| SQLStatisticsW       |
| SQLTablePrivilegesW  |