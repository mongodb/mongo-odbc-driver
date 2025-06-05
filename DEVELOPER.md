# Notes for Development

## Development Environment

### Environment Variables

You may need to set the following environment variables in order to run and test the driver with ADF locally:

| Variable                                    | Description                                                                                                                                                                                                                                                                                                                          |
|---------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ADF_TEST_LOCAL_USER | local adf username, used for integration and result set tests |
| ADF_TEST_LOCAL_PWD | local adf password, used for integration and result set tests  |
| ADF_TEST_LOCAL_AUTH_DB | local adf auth database (e.g. `admin`), used for integration and result set tests |
| ADF_TEST_LOCAL_HOST | local adf host (e.g. `localhost`), used for integration and result set tests |
| ADF_TEST_LOCAL_DB | local adf database (e.g. `integration_test`), used for integration and result set tests |
| MDB_TEST_LOCAL_PORT | local adf port (e.g. `28017`), used for integration and result set tests |


### Setting up the driver manager on MacOS

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
export DRIVER_LIB_PATH=$PWD/target/release
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

### Setting up the driver manager on Windows

In order to set up the driver on windows, we first need to update the systemDSN registry file with the environment variables we set earlier. From the `mongo-odbc-driver` folder, run:
```
sed -i 's@%DRIVER_DLL_PATH%@'"$(echo "$(cygpath -w $(pwd))" | sed s',\\,\\\\\\\\,g')"'@' setup/setupDSN.reg
sed -i 's@%ADF_TEST_USER%@'"$(echo "${ADF_TEST_LOCAL_USER}" | sed s',\\,\\\\\\\\,g')"'@' setup/setupDSN.reg
sed -i 's@%ADF_TEST_PWD%@'"$(echo "${ADF_TEST_LOCAL_PWD}" | sed s',\\,\\\\\\\\,g')"'@' setup/setupDSN.reg
sed -i 's@%ADF_TEST_URI%@'"$(echo "${ADF_TEST_URI}" | sed s',\\,\\\\\\\\,g')"'@' setup/setupDSN.reg
sed -i 's@%ADF_TEST_DB%@'"$(echo "${ADF_TEST_LOCAL_DB}" | sed s',\\,\\\\\\\\,g')"'@' setup/setupDSN.reg
```
Then, we simply need to register these keys with the system:
```
reg import "setup\setupDSN.reg"
```

## Building from Source

To build and test the driver, the standard cargo commands can be used from the root directory.

For an unoptimized build with debugging information (most common), the following will build and output build files to the `target/debug` directory:
- (windows, linux): `cargo build`
- (macos): `cargo build --features definitions/iodbc,cstr/utf32`

For an optimized build with debugging information, the following will build and output build files to the `target/release` directory:
- (windows, linux): `cargo build --release`
- (macos): `cargo build --features definitions/iodbc,cstr/utf32 --release`

## Running Tests

### To run unit tests

Similar to building, standard cargo commands can be used here:

- (windows, linux): `cargo test unit`
- (macos): `cargo test --features definitions/iodbc,cstr/utf32 unit`

### Other types of tests
The other tests that are run are integration and result set tests. These involve more setup, and that setup is operating system dependent. Regardless of the operating system, the environment variables described above must be set.

The first step on either macos or windows is to start a local mongod and Atlas Data Federation instance, and load sample data into them. A necessary prerequisite is having golang installed (see [here](https://go.dev/doc/install)).
```
./resources/run_adf.sh start
```

To load the data,  use the [sql-engines-common-test-infra](https://github.com/10gen/sql-engines-common-test-infra)
`data-loader` tool along with the data in the `resources/integration_test/testdata.tar.gz` archive. Decompress that
data and use it with the `data-loader`. See `cargo run --bin data-loader -- --help` in that repo for more details.

#### macos
To run result set sets:
```
cargo test --package integration_test --lib test_runner --features definitions/iodbc,cstr/utf32,integration_test/result_set
```
To run integration tests (note: at present, there is still work to be done to ensure these run properly. Some failures are expected):
```
cargo test  --features definitions/iodbc,cstr/utf32 integration
```

#### windows
To run result set sets:
```
cargo test --package integration_test --lib test_runner --features integration_test/result_set
```
To run integration tests (note: at present, there is still work to be done to ensure these run properly. Some failures are expected):
```
cargo test integration
```

## Evergreen

To run our suite of checks and tests against a given branch, a patch can be submitted to evergreen. The project id on evergreen is `mongosql-odbc-driver` (note the difference from the repository's name). An example command for testing your local, uncommitted changes would be:
```
evergreen patch -p mongosql-odbc-driver --uncommitted
```
More information on using evergreen can be found in the [R&D Docs](https://docs.devprod.prod.corp.mongodb.com/evergreen/Home).

## Conventions

### Panic Handler

When executing ODBC functions, we want to ensure that when a panic occurs, it is handled gracefully (caught and logged) as opposed to crashing the program. To do this, we have two macros, `panic_safe_exec_clear_diagnostics` and `panic_safe_exec_keep_diagnostics`. These both wrap the functions being used, converting any panics to errors, and subsequently managing the underlying handle's `errors` vector. As the names imply, the former clears the errors vector before adding the error generated by the panic, while the latter simply appends.

In addition, these macros take in a log level and handle tracing for successful execution.

All ODBC function calls should be wrapped in one of these two.

### Handle Assertions

Our function signatures come directly from the ODBC spec, and we use the `odbc_sys` crate to define our Foreign Function Interface (FFI). As a result, when handles are passed to our functions, they are received as pointers to `odbc_sys` types, rather than our own internal representation of these types (for example, `SQLGetDiagRec` takes a parameter of type `HEnv`, which is just a pointer to an `odbc_sys` `Env`). Thus, when we receive pointers to the handles we have allocated as arguments to functions, they need to be converted from pointers to `odbc_sys` types back to our own native types. The convention to do so is:
1. Convert the input handle to a `MongoHandleRef` using `MongoHandleRef::From(...)`
2. Convert this `MongoHandleRef` to a handle of the specific type we expect using the `must_be_<handle type>` macros.

For example, if a function takes in an environment handle `henv` of type `HEnv`:
```
let mongo_handle = MongoHandleRef::From(henv);
let env = must_be_env!(mongo_handle);
```

`env` now gives us access to our own `Env` struct and lets us work with our internal model for that directly.

Note that this applies to all handle types (`Conn`, `Desc`, `Env`, `Stmt`).
