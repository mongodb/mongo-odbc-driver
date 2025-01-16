#!/bin/bash

LICENSE_FILE_NAME="THIRD_PARTY_LICENSES.txt"

# Install the cargo-bundle-licenses tool
cargo install cargo-bundle-licenses

# Generate the license file
cargo bundle-licenses --format yaml --output resources/licenses.yaml

# Ensure a clean slate by deleting the existing THIRD_PARTY_LICENSES.txt file
rm -f "$LICENSE_FILE_NAME"

# Concatenate the third_party_header.txt file and the licenses.yaml file to create the THIRD_PARTY_LICENSES.txt file
cat resources/third_party_header.txt >"$LICENSE_FILE_NAME"
cat resources/licenses.yaml >>"$LICENSE_FILE_NAME"
