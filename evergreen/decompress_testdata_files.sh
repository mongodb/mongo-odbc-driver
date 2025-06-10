#!/bin/bash

cd resources/integration_test

echo "Removing existing testdata..."
rm -rf testdata/
echo "Removal complete"

echo "Decompressing integration test data..."
tar -xzvf testdata.tar.gz
echo "Decompression complete"
