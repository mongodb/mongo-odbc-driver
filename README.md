# mongo-odbc-driver

An ODBC driver for connecting to Atlas Data Lake via the MongoSQL dialect.

If you're looking for an ODBC driver to use with the [MongoDB
Connector for BI](https://docs.mongodb.com/bi-connector/current/),
please see the
[mongodb/mongo-bi-connector-odbc-driver](https://github.com/mongodb/mongo-bi-connector-odbc-driver)
repository.


## Set up the ODBC driver
1. If you are not on a Windows machine, install the [unixODBC](http://www.unixodbc.org/) driver manager. Windows users can utilize the built-in driver manager.
2. Update the values of `Driver`, `Pwd`, `Server`, `User`, and `Database` in `setup/setupDSN.reg`. The value of `Driver` should be the absolute path of `mongo-odbc-driver/target/debug/mongoodbc.dll`.
3. Open the registry editor and add a new key under `ODBC/ODBCINST.INI` called `ADL_ODBC`. Then, add two sub-entries to `ADL_ODBC`:
    ````
    Driver: <path to dll>
    Setup: <path to dll>
   ````
4. Run `reg import "setup/setupDSN.reg"` to populate the registry editor with the correct subkeys, entries, and values. You can run `reg query "HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI\ADL_ODBC_DRIVER"` to determine if the command ran successfully.

There should be a new entry called `ADL_TEST` under `ODBC/ODBC.INI`. If you are on Windows, open the Microsoft ODBC Administrator (64-bit) and verify that "ADL_ODBC_DRIVER" appears under "System DSN".
