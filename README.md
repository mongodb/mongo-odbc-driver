# mongo-odbc-driver

An ODBC driver for connecting to Atlas Data Lake via the MongoSQL dialect.

If you're looking for an ODBC driver to use with the [MongoDB
Connector for BI](https://docs.mongodb.com/bi-connector/current/),
please see the
[mongodb/mongo-bi-connector-odbc-driver](https://github.com/mongodb/mongo-bi-connector-odbc-driver)
repository.


## Set up the ODBC driver on Windows (architecture-agnostic)
Note: users can utilize the built-in driver manager.
1. Update the values of `Driver`, `Pwd`, `Server`, `User`, and `Database` in `setup/setupDSN.reg`. The value of `Driver` should be the absolute path of `mongo-odbc-driver/target/debug/mongoodbc.dll`.
2. Run `reg import "setup/setupDSN.reg"` in order to populate the registry editor with the new entries. Alternatively, simply double click on the `setupDSN.reg` file.

## Validate setup
### 64-bit
Run `reg query "HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers"` to verify that `ADL_ODBC_DRIVER` has been installed successfully.

There should be a new entry called `ADL_TEST` under `ODBC/ODBC.INI` with the following subentries:

	````
	database: <database name>
	pwd: <password>
	server: <server>
	user: <user>
	````

Run `reg query "HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI\ADL_ODBC_DRIVER"` to determine if the registry editor was updated successfully. There should also be a new entry called `ADL_ODBC` under `ODBC/ODBCINST.INI` with the following subentries:

    ````
    Driver: <path to dll>
    Setup: <path to dll>
   ````

Open the Microsoft ODBC Administrator (64-bit) and verify that "ADL_ODBC_DRIVER" appears under "System DSN".

### 32-bit
Follow the validation steps listed in the 64-bit section, but make sure to use the 32-bit Microsoft ODBC Administrator. Further modifications: the registry keys should be listed under `HKEY_LOCAL_MACHINE\SOFTWARE\Wow6432Node\ODBC` instead of `HKEY_LOCAL_MACHINE\SOFTWARE\ODBC`, and the dll path must point to the 32-bit version of the driver.