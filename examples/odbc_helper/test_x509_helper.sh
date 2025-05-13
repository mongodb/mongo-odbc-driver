#!/bin/bash
echo "Testing x509_odbc_helper with different certificate path configurations..."

# Test with help flag
echo -e "\n1. Testing help flag:"
./x509_odbc_helper --help

# Test with certificate paths as command-line arguments
echo -e "\n2. Testing with certificate paths as command-line arguments:"
./x509_odbc_helper "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/?authSource=$external&authMechanism=MONGODB-X509;" "SELECT * FROM test LIMIT 1" "/tmp/test-client.pem" "/tmp/test-ca.pem" 2>&1 | grep "connection string"

# Test with certificate paths in the connection string
echo -e "\n3. Testing with certificate paths in the connection string:"
./x509_odbc_helper "Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/?authSource=$external&authMechanism=MONGODB-X509;sslClientCertificateKeyFile=/tmp/test-client.pem;sslCAFile=/tmp/test-ca.pem;" 2>&1 | grep "connection string"

echo -e "\nAll tests completed."
