# mongo-odbc-driver

An ODBC driver for connecting to Atlas Data Lake via the MongoSQL dialect.

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

### 32-bit
Follow the validation steps listed in the 64-bit section, but make sure to use the 32-bit Microsoft ODBC Administrator. Additionally, the registry keys should be listed under `HKEY_LOCAL_MACHINE\SOFTWARE\Wow6432Node\ODBC` instead of `HKEY_LOCAL_MACHINE\SOFTWARE\ODBC`, and the dll path must point to the 32-bit version of the driver.

\<document field access expression\> ::= [\<compound identifier\>](#identifiers) \| [\<expression\>](#expressions) \".\" [\<compound identifier\>](#identifiers)</br>

## Unsupported Functions

The following ODBC functions are currently not supported by the driver. Note that some unsupported ANSI functions have Unicode analogs. If the version of the function suffixed with 'W', e.g., `SQLTablesW`, is not mentioned, the Unicode version is either supported or may be supported in the future.

| function |
| --- |
| SQLBindParameter |
| SQLBrowseConnect |
| SQLBulkOperations |
| SQLColAttribute |
| SQLColumnPrivileges |
| SQLCompleteAsync |
| SQLConnect |
| SQLConnectW |
| SQLCopyDesc |
| SQLDataSources |
| SQLDataSourcesW |
| SQLDescribeCol |
| SQLDescribeParam |
| SQLDrivers |
| SQLDriversW |
| SQLExecute |
| SQLGetConnectAttr |
| SQLGetCursorName |
| SQLGetDescField |
| SQLGetDescFieldW |
| SQLGetDescRec |
| SQLGetDescRecW |
| SQLGetDiagField |
| SQLGetEnvAttr |
| SQLGetInfo |
| SQLGetStmtAttr |
| SQLNativeSql |
| SQLNumParams |
| SQLParamData |
| SQLPrepare |
| SQLPrepareW |
| SQLProcedureColumns |
| SQLProcedureColumnsW |
| SQLProcedures |
| SQLProceduresW |
| SQLPutData |
| SQLSetConnectAttr |
| SQLSetCursorName |
| SQLSetDescField |
| SQLSetPos |
| SQLSetStmtAttr |
| SQLSpecialColumns |
| SQLStatistics |
| SQLTablePrivileges |
| SQLTables |