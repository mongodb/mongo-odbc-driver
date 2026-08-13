#!/bin/bash
set -e


function download {
  echo "Downloading artifact $1"
  curl -LO "$1" \
  --silent \
  --fail \
  --max-time 600 \
  --retry 5 \
  --retry-delay 0
}

# Download the sample_airbnb data and schemas
download "https://mongosql-noexpire.s3.us-east-2.amazonaws.com/ODBC_driver_test_data/AlexiCluster0SampleAirbnb.archive.gz"

case ${build_variant} in
  ubuntu2204)
    echo "Setting up mongodb-database-tools and mongosh for Ubuntu 22.04 x86_64"

    MONGO_TOOLS_DOWNLOAD="mongodb-database-tools-ubuntu2204-x86_64-100.17.0"
    MONGOSH_DOWNLOAD="mongosh-2.10.0-linux-x64"
    ;;

  ubuntu2204-arm64)
    echo "Setting up mongodb-database-tools and mongosh for Ubuntu 22.04 ARM64"

    MONGO_TOOLS_DOWNLOAD="mongodb-database-tools-ubuntu2204-arm64-100.17.0"
    MONGOSH_DOWNLOAD="mongosh-2.10.0-linux-arm64"
    ;;

  *)
    echo "Unknown build_variant: ${build_variant}"
    exit 1
    ;;
esac

# Download and extract the mongodb-database-tools package
download "https://fastdl.mongodb.org/tools/db/$MONGO_TOOLS_DOWNLOAD.tgz"
tar zxvf "$MONGO_TOOLS_DOWNLOAD".tgz

# Download and extract mongosh
download "https://downloads.mongodb.com/compass/$MONGOSH_DOWNLOAD.tgz"
tar zxvf "$MONGOSH_DOWNLOAD".tgz

# Create on prem test user
"$MONGOSH_DOWNLOAD"/bin/mongosh admin --port 28017 --eval "db.createUser({user: '${on_prem_test_user}', pwd: '${on_prem_test_pwd}', roles: ['readWrite']})"

# Run `mongorestore`
"$MONGO_TOOLS_DOWNLOAD"/bin/mongorestore --drop --numInsertionWorkersPerCollection=8 --bypassDocumentValidation --gzip --port=28017 --archive=AlexiCluster0SampleAirbnb.archive.gz