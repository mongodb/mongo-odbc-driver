# Notes for Development

## Development Environment

## Building from Source

To build and test the driver, the standard cargo commands can be used from the root directory.

For an unoptimized build with debugging information (most common):
```
cargo build
```

For an optimized build with debugging info (mac)
```
cargo build --features odbc-sys/iodbc,cstr/utf32 --profile=release-with-debug
```
And for an optimized build with debugging info (windows, linux)
```
cargo build --profile=release-with-debug
```
The resulting build files will be output to the `target` directory in the `debug` or `release` folders respectively.

## Running Tests

### To run unit tests

Similar to building, standard cargo commands can be used here:

```
cargo test unit
```

## Evergreen

To run our suite of checks and tests against a given branch, a patch can be submitted to evergreen. The project id on evergreen is `mongosql-odbc-driver` (note the difference from the repository's name). An example command for testing your local, uncommited changes would be:
```
evergreen patch -p mongosql-odbc-driver --uncommitted
```
More information on using evergreen can be found in the [R&D Docs](https://docs.devprod.prod.corp.mongodb.com/evergreen/Home).
