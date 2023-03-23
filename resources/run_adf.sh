#!/bin/bash
# 
# Usage: run_adf.sh <operation>
# operation: 'start' or 'stop'
#
# This script will start a local mongod and Atlas Data Federation instance, used for integration testing.
# The supported platforms are windows, macos, ubuntu1804, and rhel7.
#
# - To skip the download of ADF, set the environment variable HAVE_LOCAL_MONGOHOUSE to 1
#   and set the environment variable LOCAL_MONGOHOUSE_DIR to the root directory of the
#   mongohouse source tree.
# - To skip the operations of this script, set the environment variable SKIP_RUN_ADF to 1.

NAME=`basename "$0"`
if [[ $SKIP_RUN_ADF -eq 1 ]]; then
  echo "Skipping $NAME"
  exit 0
fi

ARG=`echo $1 | tr '[:upper:]' '[:lower:]'`
if [[ -z $ARG ]]; then
  echo "Usage: $NAME <operation>"
  echo "operation: 'start' or 'stop'"
  exit 0
fi

GO_VERSION="go1.18"
if [ -d "/opt/golang/$GO_VERSION" ]; then
  GOROOT="/opt/golang/$GO_VERSION"
  GOBINDIR="$GOROOT"/bin
elif [ -d "C:\\golang\\$GO_VERSION" ]; then
  GOROOT="C:\\golang\\$GO_VERSION"
  GOBINDIR="$GOROOT"\\bin
  export GOCACHE=$(cygpath -m $HOME/gocache)
  export GOPATH=$(cygpath -m $HOME/go)
else #local testing
  GOBINDIR=/usr/bin
fi

GO="$GOBINDIR/go"

PATH=$GOBINDIR:$PATH

LOCAL_INSTALL_DIR=$(pwd)/local_adf
MONGOHOUSE_URI=git@github.com:10gen/mongohouse.git
MONGO_DB_PATH=$LOCAL_INSTALL_DIR/test_db
LOGS_PATH=$LOCAL_INSTALL_DIR/logs
DB_CONFIG_PATH=$(pwd)/resources/integration_test/testdata/adf_db_config.json
MONGOD_PORT=28017
MONGOHOUSED_PORT=27017
START="start"
STOP="stop"
MONGOD="mongod"
MONGOHOUSED="mongohoused"
TENANT_CONFIG="./testdata/config/mongodb_local/tenant-config.json"
MONGO_DOWNLOAD_LINK=
MONGO_DOWNLOAD_DIR=
MONGO_DOWNLOAD_FILE=
OS=$(uname)
if [[ $OS =~ ^CYGWIN ]]; then
    TMP_DIR="C:\\temp\\run_adf"
else
    TMP_DIR="/tmp/run_adf"
fi
TIMEOUT=180
JQ=$TMP_DIR/jq


## OS Agnostic definitions
MONGO_DOWNLOAD_BASE=https://fastdl.mongodb.org
MONGOSH_DOWNLOAD_BASE=https://downloads.mongodb.com/compass

## Linux
# Ubuntu 22.04
MONGO_DOWNLOAD_UBUNTU=mongodb-linux-x86_64-ubuntu2204-6.0.4.tgz
# RedHat 7
MONGO_DOWNLOAD_REDHAT=mongodb-linux-x86_64-rhel70-6.0.4.tgz
# Shared Linux mongosh
MONGOSH_DOWNLOAD_LINUX_FILE=mongosh-1.8.0-linux-x64.tgz

## macOS
MONGO_DOWNLOAD_MAC=mongodb-macos-x86_64-6.0.4.tgz
MONGOSH_DOWNLOAD_MAC_FILE=mongosh-1.8.0-darwin-x64.zip

## Windows
MONGO_DOWNLOAD_WIN=mongodb-windows-x86_64-6.0.4.zip
MONGOSH_DOWNLOAD_WINDOWS_FILE=mongosh-1.8.0-win32-x64.zip

mkdir -p $LOCAL_INSTALL_DIR

check_procname() {
  ps -ef 2>/dev/null | grep $1 | grep -v grep >/dev/null
  result=$?

  if [[ result -eq 0 ]]; then
    return 0
  else
    return 1
  fi
}

check_version() {
  VERSION=`$MONGOSH_DOWNLOAD_DIR/bin/mongosh --port $1 --eval 'db.version()'`
  result=$?
  VERSION=$(echo $VERSION | tail -n 1)
  echo "check_version() output"
  echo $VERSION

  if [[ result -eq 0 ]]; then
    return 0
  else
    return 1
  fi
}

check_mongod() {
  check_procname $MONGOD
  process_check_result=$?
  check_version $MONGOD_PORT
  port_check_result=$?

  if [[ $process_check_result -eq 0 ]] && [[ $port_check_result -eq 0 ]]; then
    return 0
  else
    return 1
  fi
}

# Check if jq exists.  If not, download and set path
get_jq() {
  if [ $OS = "Linux" ]; then
    curl -L -o $JQ https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64
  elif [ $OS = "Darwin" ]; then
    curl -L -o $JQ https://github.com/stedolan/jq/releases/download/jq-1.6/jq-osx-amd64
  else
    curl -L -o $JQ https://github.com/stedolan/jq/releases/download/jq-1.6/jq-win64.exe
  fi
  chmod +x $JQ
}

check_mongohoused() {
  check_version $MONGOHOUSED_PORT
  return $?
}

# check if mac or linux
if [ $OS = "Linux" ]; then
  distro=$(awk -F= '/^NAME/{print $2}' /etc/os-release)
  if [ "$distro" = "\"Red Hat Enterprise Linux\"" ] ||
[ "$distro" = "\"Red Hat Enterprise Linux Server\"" ]; then
    export VARIANT=rhel7
    MONGO_DOWNLOAD_LINK=$MONGO_DOWNLOAD_BASE/linux/$MONGO_DOWNLOAD_REDHAT
    MONGO_DOWNLOAD_FILE=$MONGO_DOWNLOAD_REDHAT
    MONGOSH_DOWNLOAD_FILE=$MONGOSH_DOWNLOAD_LINUX_FILE
    MONGOSH_DOWNLOAD_LINK=$MONGOSH_DOWNLOAD_BASE/$MONGOSH_DOWNLOAD_FILE
  elif [ "$distro" = "\"Ubuntu\"" ]; then
    export VARIANT=ubuntu2204
    MONGO_DOWNLOAD_LINK=$MONGO_DOWNLOAD_BASE/linux/$MONGO_DOWNLOAD_UBUNTU
    MONGO_DOWNLOAD_FILE=$MONGO_DOWNLOAD_UBUNTU
    MONGOSH_DOWNLOAD_FILE=$MONGOSH_DOWNLOAD_LINUX_FILE
    MONGOSH_DOWNLOAD_LINK=$MONGOSH_DOWNLOAD_BASE/$MONGOSH_DOWNLOAD_FILE
  else
    echo ${distro} not supported
    exit 1
  fi
elif [ $OS = "Darwin" ]; then
  MONGO_DOWNLOAD_LINK=$MONGO_DOWNLOAD_BASE/osx/$MONGO_DOWNLOAD_MAC
  MONGO_DOWNLOAD_FILE=$MONGO_DOWNLOAD_MAC
  MONGOSH_DOWNLOAD_FILE=$MONGOSH_DOWNLOAD_MAC_FILE
  MONGOSH_DOWNLOAD_LINK=$MONGOSH_DOWNLOAD_BASE/$MONGOSH_DOWNLOAD_FILE
elif [[ $OS =~ ^CYGWIN ]]; then
  MONGO_DOWNLOAD_LINK=$MONGO_DOWNLOAD_BASE/windows/$MONGO_DOWNLOAD_WIN
  MONGO_DOWNLOAD_FILE=$MONGO_DOWNLOAD_WIN
  MONGOSH_DOWNLOAD_FILE=$MONGOSH_DOWNLOAD_WINDOWS_FILE
  MONGOSH_DOWNLOAD_LINK=$MONGOSH_DOWNLOAD_BASE/$MONGOSH_DOWNLOAD_FILE
else
  echo $(uname) not supported
  exit 1
fi

install_mongodb() {
    (cd $LOCAL_INSTALL_DIR && curl -O $MONGO_DOWNLOAD_LINK)
    if [[ $OS =~ ^CYGWIN ]]; then
      unzip -qo $LOCAL_INSTALL_DIR/$MONGO_DOWNLOAD_FILE -d $LOCAL_INSTALL_DIR 2> /dev/null

      # Obtain unzipped directory name
      MONGO_UNZIP_DIR=$(unzip -lq $LOCAL_INSTALL_DIR/$MONGO_DOWNLOAD_FILE | grep mongod.exe | tr -s ' ' \
	          | cut -d ' ' -f 5 | cut -d/ -f1)
      chmod -R +x $LOCAL_INSTALL_DIR/$MONGO_UNZIP_DIR/bin/
      echo $LOCAL_INSTALL_DIR/$MONGO_UNZIP_DIR
    else
      tar zxf $LOCAL_INSTALL_DIR/$MONGO_DOWNLOAD_FILE --directory $LOCAL_INSTALL_DIR
      echo $LOCAL_INSTALL_DIR/${MONGO_DOWNLOAD_FILE:0:$((${#MONGO_DOWNLOAD_FILE} - 4))}
    fi
}

install_mongosh() {
    (cd $LOCAL_INSTALL_DIR && curl -O $MONGOSH_DOWNLOAD_LINK)
    if [[ $OS =~ ^CYGWIN ]]; then
      unzip -qo $LOCAL_INSTALL_DIR/$MONGOSH_DOWNLOAD_FILE -d $LOCAL_INSTALL_DIR 2> /dev/null

      # Obtain unzipped directory name
      MONGOSH_UNZIP_DIR=$(unzip -lq $LOCAL_INSTALL_DIR/$MONGOSH_DOWNLOAD_FILE | grep mongosh.exe | tr -s ' ' \
	          | cut -d ' ' -f 5 | cut -d/ -f1)
      chmod -R +x $LOCAL_INSTALL_DIR/$MONGOSH_UNZIP_DIR/bin/
      echo $LOCAL_INSTALL_DIR/$MONGOSH_UNZIP_DIR
    else
      tar zxf $LOCAL_INSTALL_DIR/$MONGOSH_DOWNLOAD_FILE --directory $LOCAL_INSTALL_DIR
      echo $LOCAL_INSTALL_DIR/${MONGOSH_DOWNLOAD_FILE:0:$((${#MONGOSH_DOWNLOAD_FILE} - 4))}
    fi
}

MONGO_DOWNLOAD_DIR=$(install_mongodb)
MONGOSH_DOWNLOAD_DIR=$(install_mongosh)

check_mongod $MONGO_DOWNLOAD_DIR
if [[ $? -ne 0 ]]; then
  if [ $ARG = $START ]; then
    echo "Starting $MONGOD"

    mkdir -p $MONGO_DB_PATH
    mkdir -p $LOGS_PATH
    mkdir -p $TMP_DIR

    # Start mongod
    (cd $LOCAL_INSTALL_DIR && curl -O $MONGO_DOWNLOAD_LINK)

    # Note: ADF has a storage.json file that generates configs for us.
    # The mongodb source is on port $MONGOD_PORT so we use that here.
    # Uncompress the archive
    if [[ $OS =~ ^CYGWIN ]]; then
      # mongod does not have --fork option on Windows, using nohup
      nohup $MONGO_DOWNLOAD_DIR/bin/mongod --port $MONGOD_PORT --dbpath $(cygpath -m ${MONGO_DB_PATH}) \
              --logpath $(cygpath -m $LOGS_PATH/mongodb_test.log) &
      echo $! > $TMP_DIR/${MONGOD}.pid
    else
      $MONGO_DOWNLOAD_DIR/bin/mongod --port $MONGOD_PORT --dbpath $MONGO_DB_PATH \
        --logpath $LOGS_PATH/mongodb_test.log --pidfilepath $TMP_DIR/${MONGOD}.pid --fork
    fi

    waitCounter=0
    while : ; do
        check_mongod 
        if [[ $? -eq 0 ]]; then
            break
        fi
        if [[ "$waitCounter" -gt $TIMEOUT ]]; then
            echo "ERROR: Local mongod did not start under $TIMEOUT seconds"
            exit 1
        fi
        let waitCounter=waitCounter+1
        sleep 1
    done
  fi
else
  if [ $ARG = $STOP ]; then
    MONGOD_PID=$(< $TMP_DIR/${MONGOD}.pid)
    echo "Stopping $MONGOD, pid $MONGOD_PID"
    kill "$(< $TMP_DIR/${MONGOD}.pid)"
  fi
fi

check_mongohoused
if [[ $? -ne 0 ]]; then
  if [ $ARG = $START ]; then
    echo "Starting $MONGOHOUSED"
    $GO version

    if [[ $HAVE_LOCAL_MONGOHOUSE -eq 1 ]]; then
        if [ ! -d "$LOCAL_MONGOHOUSE_DIR" ]; then
            echo "ERROR: $LOCAL_MONGOHOUSE_DIR is not a directory"
            exit 1
        fi
        cd $LOCAL_MONGOHOUSE_DIR
    else
        echo "Downloading mongohouse"
        if [[ $OS =~ ^CYGWIN ]]; then
          MONGOHOUSE_DIR=$(cygpath -m $LOCAL_INSTALL_DIR/mongohouse)
        else
          MONGOHOUSE_DIR=$LOCAL_INSTALL_DIR/mongohouse
        fi
        # Install and start mongohoused
        git config --global url.git@github.com:.insteadOf https://github.com/
        # Clone the mongohouse repo
        if [ ! -d "$MONGOHOUSE_DIR" ]; then
            git clone $MONGOHOUSE_URI $MONGOHOUSE_DIR
        fi
        cd $MONGOHOUSE_DIR
        git pull $MONGOHOUSE_URI

        export GOPRIVATE=github.com/10gen
        $GO mod download
    fi

    # Set relevant environment variables
    export MONGOHOUSE_ENVIRONMENT="local"
    if [[ $OS =~ ^CYGWIN ]]; then
      export MONGOHOUSE_MQLRUN=$(cygpath -m $(pwd)/artifacts/mqlrun.exe)
      export LIBRARY_PATH=$(cygpath -m $(pwd)/artifacts)
      MONGOSQL_LIB=$(cygpath -m $(pwd)/mongosql.dll)
    else
      export MONGOHOUSE_MQLRUN="$(pwd)/artifacts/mqlrun"
      export LIBRARY_PATH="$(pwd)/artifacts"
      MONGOSQL_LIB=$(pwd)/artifacts/libmongosql.a
    fi

    # Download latest versions of external dependencies
    if [[ $HAVE_LOCAL_MONGOHOUSE -eq 1 && -f "$MOGOHOUSE_MQLRUN" ]]; then
        cp ${MONGOHOUSE_MQLRUN} ${MOGOHOUSE_MQLRUN}.orig
    fi
    if [[ $HAVE_LOCAL_MONGOHOUSE -eq 1 && -f "$MONGOSQL_LIB" ]]; then
        cp ${MONGOSQL_LIB} ${MONGOSQL_LIB}.orig
    fi
    rm -f $MONGOHOUSE_MQLRUN
    $GO run cmd/buildscript/build.go tools:download:mqlrun
    rm -f $MONGOSQL_LIB
    $GO run cmd/buildscript/build.go tools:download:mongosql

    get_jq
    # Load tenant config into mongodb
    STORES='{ "name" : "localmongo", "provider" : "mongodb", "uri" : "mongodb://localhost:%s" }'
    STORES=$(printf "$STORES" "${MONGOD_PORT}")
    DATABASES=$(cat $DB_CONFIG_PATH)

    # Replace the existing storage config with a wildcard collection for the local mongodb
    cp ${TENANT_CONFIG} ${TENANT_CONFIG}.orig
    $JQ "del(.storage)" ${TENANT_CONFIG} > ${TENANT_CONFIG}.tmp && mv ${TENANT_CONFIG}.tmp ${TENANT_CONFIG}
    $JQ --argjson obj "$STORES" '.storage.stores += [$obj]' ${TENANT_CONFIG} > ${TENANT_CONFIG}.tmp\
                                                               && mv ${TENANT_CONFIG}.tmp ${TENANT_CONFIG}
    $JQ --argjson obj "$DATABASES" '.storage.databases += $obj' ${TENANT_CONFIG} > ${TENANT_CONFIG}.tmp\
                                                               && mv ${TENANT_CONFIG}.tmp ${TENANT_CONFIG}

    $GO run cmd/buildscript/build.go init:mongodb-tenant

    mkdir -p $TMP_DIR
    mkdir -p $LOGS_PATH
    # Start mongohoused with appropriate config
    nohup $GO run -tags mongosql ./cmd/mongohoused/mongohoused.go \
      --config ./testdata/config/mongodb_local/frontend-agent-backend.yaml >> $LOGS_PATH/${MONGOHOUSED}.log &
    echo $! > $TMP_DIR/${MONGOHOUSED}.pid

    waitCounter=0
    while : ; do
        check_mongohoused
        if [[ $? -eq 0 ]]; then
            break
        fi
        if [[ "$waitCounter" -gt $TIMEOUT ]]; then
            echo "ERROR: Local ADF did not start under $TIMEOUT seconds"
            exit 1
        fi
        let waitCounter=waitCounter+1
        sleep 1
    done
  fi
fi
if [ $ARG = $STOP ]; then
  MONGOHOUSED_PID=$(< $TMP_DIR/${MONGOHOUSED}.pid)
  echo "Stopping $MONGOHOUSED, pid $MONGOHOUSED_PID"

  if [[ $OS =~ ^CYGWIN ]]; then
    ps -W | grep $MONGOHOUSED | sed 's/   */:/g' | cut -d: -f5 | xargs -l taskkill /F /PID
  else
    pkill -TERM -P ${MONGOHOUSED_PID}
  fi
  if [[ $HAVE_LOCAL_MONGOHOUSE -eq 1 && -d "$LOCAL_MONGOHOUSE_DIR" ]]; then
      echo "Restoring ${TENANT_CONFIG}"
      cd $LOCAL_MONGOHOUSE_DIR
      mv ${TENANT_CONFIG}.orig ${TENANT_CONFIG}
      if [[ -f ${MONGOSQL_MQLRUN}.orig ]] ; then
          echo "Restoring $MONGOSQL_MQLRUN"
          mv ${MONGOSQL_MQLRUN}.orig $MONGOSQL_MQLRUN
      fi
      MONGOSQL_LIB=$LOCAL_MONGOHOUSE_DIR/artifacts/libmongosql.a
      if [[ -f ${MONGOSQL_LIB}.orig ]] ; then
          echo "Restoring $MONGOSQL_LIB"
          mv ${MONGOSQL_LIB}.orig $MONGOSQL_LIB
      fi
  fi
fi

