# Notes for Development

## Development Environment

## Building from Source

To build and test the driver, the standard cargo commands can be used from the root directory.

For an unoptimized build with debugging information (most common), the following will build and output build files to the `target/debug` directory:
- (windows, linux): `cargo build`
- (macos): `cargo build --features odbc-sys/iodbc,cstr/utf32`

For an optimized build with debugging information, the following will build and output build files to the `target/release-with-debug` directory:
- (windows, linux): `cargo build --profile=release-with-debug`
- (macos): `cargo build --features odbc-sys/iodbc,cstr/utf32 --profile=release-with-debug`

## Setting up the driver manager on MacOS

For macos, we use [iodbc](https://www.iodbc.org/dataspace/doc/iodbc/wiki/iodbcWiki/WelcomeVisitors) as the driver manager. To set up iODBC, first download and install it using the following commands:
```
export $INSTALLED_ODBC_PATH=$PWD/installed_odbc/install"
mkdir -p "$INSTALLED_ODBC_PATH"
cd installed_odbc
echo "downloading iODBC"
iODBC_dir=libiodbc-3.52.15
curl -LO "https://github.com/openlink/iODBC/releases/download/v3.52.15/$iODBC_dir.tar.gz" \
--silent \
--fail \
--max-time 60 \
--retry 5 \
--retry-delay 0

tar xf "$iODBC_dir.tar.gz"
cd "$iODBC_dir"
./configure --prefix="$INSTALLED_ODBC_PATH"
make 
make install
```
Next, using the steps in [Building From Source](#building-from-source), compile the release version with debug info. Next, set the various library paths:
```
export LIBRARY_PATH="$INSTALLED_ODBC_PATH/lib"
export LD_LIBRARY_PATH="$INSTALLED_ODBC_PATH/lib"
export DYLD_LIBRARY_PATH="$INSTALLED_ODBC_PATH/lib"
```
Finally, return to the root folder, and update the ini files to set the relevant values and export additional environment variables:
```
export DRIVER_LIB_PATH=$PWD/target/release-with-debug
sed -i.bu "s,%DRIVER_LIB_PATH%,$DRIVER_LIB_PATH,g" setup/iodbcinst.ini
sed -i.bu "s,%DRIVER_LIB_PATH%,$DRIVER_LIB_PATH,g" setup/iodbc.ini
sed -i.bu "s,%ADF_TEST_DB%,${ADF_TEST_LOCAL_DB},g" setup/iodbc.ini
sed -i.bu "s,%ADF_TEST_USER%,${ADF_TEST_LOCAL_USER},g" setup/iodbc.ini
sed -i.bu "s,%ADF_TEST_PWD%,${ADF_TEST_LOCAL_PWD},g" setup/iodbc.ini
sed -i.bu "s,%ADF_TEST_HOST%,${ADF_TEST_LOCAL_HOST},g" setup/iodbc.ini
export ODBCINSTINI="$ODBCSYSINI/iodbcinst.ini"
export ODBCINI="$ODBCSYSINI/iodbc.ini"
```
Once this is done, the driver should be properly set up with the driver manager.

## Running Tests

### To run unit tests

Similar to building, standard cargo commands can be used here:

- (windows, linux): `cargo test unit`
- (macos): `cargo test --features odbc-sys/iodbc,cstr/utf32 unit`

### Other types of tests
The other tests that are run are integration and result set tests. These involve more setup, and that setup is operating system dependent. Regardless of the operating system, the below environment variables must be set. Following this are subsections describing the OS specific testing steps.

```
ADF_TEST_LOCAL_USER: local adf username
ADF_TEST_LOCAL_PWD: local adf password
ADF_TEST_LOCAL_AUTH_DB: local adf auth database (e.g. `admin`)
ADF_TEST_LOCAL_HOST: local adf host (e.g. `localhost`)
ADF_TEST_LOCAL_DB: local adf database
MDB_TEST_LOCAL_PORT: local adf port
```

#### macos
First, start a local mongod and Atlas Data Federation instance, and load sample data into them. A necessary prerequisite is having golang installed (see [here](https://go.dev/doc/install))
```
./resources/run_adf.sh start
cargo run --bin data_loader
```

To run result set sets:
```
cargo test  --features odbc-sys/iodbc,cstr/utf32 -- --ignored
```
To run integration tests (note: at present, there is still work to be done to ensure these run properly. Some failures are expected):
```
cargo test  --features odbc-sys/iodbc,cstr/utf32 integration
```

#### windows

## Evergreen

To run our suite of checks and tests against a given branch, a patch can be submitted to evergreen. The project id on evergreen is `mongosql-odbc-driver` (note the difference from the repository's name). An example command for testing your local, uncommited changes would be:
```
evergreen patch -p mongosql-odbc-driver --uncommitted
```
More information on using evergreen can be found in the [R&D Docs](https://docs.devprod.prod.corp.mongodb.com/evergreen/Home).
