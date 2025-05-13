

CONNECTION_STRING="Driver={MongoDB ODBC Driver};URI=mongodb://localhost:27017/"

echo "Testing MongoDB Advanced ODBC Helper examples..."
echo "================================================"

echo -e "\n1. Testing connection_info_utility..."
./connection_info_utility "$CONNECTION_STRING"

echo -e "\n2. Testing error_analyzer..."
./error_analyzer "$CONNECTION_STRING"

echo -e "\n3. Testing metadata_explorer (non-interactive mode)..."
echo -e "\n\n" | ./metadata_explorer "$CONNECTION_STRING"

echo -e "\n4. Testing data_type_handler..."
./data_type_handler "$CONNECTION_STRING" "system.version"

echo -e "\n5. Testing mongodb_x509_connector (without certificates)..."
./mongodb_x509_connector -u "mongodb://localhost:27017/" -q "SELECT * FROM system.version"

echo -e "\nAll tests completed."
echo "For X.509 authentication tests, use:"
echo "./mongodb_x509_connector -c /path/to/client.pem -a /path/to/ca.pem"

echo -e "\nFor interactive metadata exploration, use:"
echo "./metadata_explorer"
