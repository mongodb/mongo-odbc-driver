# common test infra begin
if [ "Windows_NT" == "$OS" ]; then
    export PROJECT_DIRECTORY=$(cygpath -m "$(pwd)")
else
    export PROJECT_DIRECTORY="$(pwd)"
fi
export COMMON_TEST_INFRA_DIR="$PROJECT_DIRECTORY/sql-engines-common-test-infra"
export DRIVERS_TOOLS="$PROJECT_DIRECTORY/evergreen/drivers-tools"
export MONGO_ORCHESTRATION_HOME="$DRIVERS_TOOLS/.evergreen/orchestration"
export MONGODB_BINARIES="$DRIVERS_TOOLS/mongodb/bin"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
# common test infra end
export INSTALLED_ODBC_PATH="$PWD/installed_odbc/install"
export LD_LIBRARY_PATH="$INSTALLED_ODBC_PATH/lib"
if [ "Windows_NT" == "$OS" ]; then
    export CARGO_BIN="$HOME/.rustup/bin:$HOME/.cargo/bin"
    export PATH="/cygdrive/c/cmake/bin:$CARGO_BIN:$LD_LIBRARY_PATH:$PATH"
    echo "------------------------------c"
    ls "/cygdrive/c"
    echo "------------------------------PF"
    ls "/cygdrive/c/Program Files"
    echo "------------------------------PFx86"
    ls "/cygdrive/c/Program Files (x86)"
else
    export CARGO_BIN="$HOME/.cargo/bin"
    export PATH="$CARGO_BIN:$LD_LIBRARY_PATH:$PATH"
fi
export DUMP_FOLDER=dumps
export LOCAL_DUMP_ORIGINAL_REG_VAL=local_dump_original_value.reg
export MONGOODBC_DEBUGGING_INFO_ARCHIVE=crashDebuggingInfo
export SCRIPT_FOLDER=resources
export COMPLIANCE_REPORT_NAME="mongo-odbc-driver_compliance_report.md"
export STATIC_CODE_ANALYSIS_NAME="mongo-odbc-driver.sast.sarif"
export FEATURE_FLAGS=""
export PRODUCT_NAME="mongoodbc"
export PATH_PREFIX=""

echo "snapshot-eap: ${snapshot-eap}"

if [[ "${triggered_by_git_tag}" != "" ]]; then
    export release_version=$(echo ${triggered_by_git_tag} | sed s/v//)

    # Check if this is a beta tag or snapshot-eap is set to true
    if [[ "${triggered_by_git_tag}" == *"beta"* || "${snapshot-eap}" == "true" ]]; then
        export FEATURE_FLAGS="eap"
        export PRODUCT_NAME="mongoodbc-eap"
        export PATH_PREFIX="eap/"
    fi
else
    # If not a tag, we are in a snapshot build. We need to see if we're in beta mode or not
    # and set the release version to either snapshot or snapshot-eap
    if [[ "${snapshot-eap}" == "true" ]]; then
        export release_version="snapshot-eap"
        export FEATURE_FLAGS="eap"
        export PRODUCT_NAME="mongoodbc-eap"
        echo "Building EAP version"
    else
        export release_version="snapshot"
    fi
fi

export MSI_FILENAME="$PRODUCT_NAME-$release_version.msi"
export UBUNTU_FILENAME="$PRODUCT_NAME-$release_version.tar.gz"

cat <<EOT >expansions.yml
release_version: "$release_version"
FEATURE_FLAGS: "$FEATURE_FLAGS"
PATH_PREFIX: "$PATH_PREFIX"
PRODUCT_NAME: "$PRODUCT_NAME"
MSI_FILENAME: "$MSI_FILENAME"
UBUNTU_FILENAME: "$UBUNTU_FILENAME"
WINDOWS_INSTALLER_PATH: "mongosql-odbc-driver/windows/$release_version/release/$MSI_FILENAME"
UBUNTU2204_INSTALLER_PATH: "mongosql-odbc-driver/ubuntu2204/$release_version/release/$UBUNTU_FILENAME"
COMPLIANCE_REPORT_NAME: "$COMPLIANCE_REPORT_NAME"
STATIC_CODE_ANALYSIS_NAME: "$STATIC_CODE_ANALYSIS_NAME"
PROJECT_DIRECTORY: "$(pwd)"
DRIVERS_TOOLS: "$DRIVERS_TOOLS"
cargo_bin: "$CARGO_BIN"
common_test_infra_dir: "$COMMON_TEST_INFRA_DIR"
skip_machete_build: "true"
script_dir: "$COMMON_TEST_INFRA_DIR/evergreen/scripts"
working_dir: "mongosql-odbc-driver"
MONGO_ORCHESTRATION_HOME: "$DRIVERS_TOOLS/.evergreen/orchestration"
MONGODB_BINARIES: "$MONGODB_BINARIES"
prepare_shell: |
  set -o errexit
  export release_version="$release_version"
  export FEATURE_FLAGS="$FEATURE_FLAGS"
  export PATH_PREFIX="$PATH_PREFIX"
  export PRODUCT_NAME="$PRODUCT_NAME"
  export MSI_FILENAME="$MSI_FILENAME"
  export UBUNTU_FILENAME="$UBUNTU_FILENAME"
  export WINDOWS_INSTALLER_PATH="$WINDOWS_INSTALLER_PATH"
  export UBUNTU2204_INSTALLER_PATH="$UBUNTU2204_INSTALLER_PATH"
  export PATH="$PATH"
  export CARGO_NET_GIT_FETCH_WITH_CLI="$CARGO_NET_GIT_FETCH_WITH_CLI"
  export LOCAL_MDB_PORT_COM=${local_mdb_port_com}
  export LOCAL_MDB_PORT_ENT=${local_mdb_port_ent}
  export LOCAL_MDB_USER=${local_mdb_user}
  export LOCAL_MDB_PWD=${local_mdb_pwd}
  export ADF_TEST_LOCAL_USER="${adf_test_local_user}"
  export ADF_TEST_LOCAL_PWD="${adf_test_local_pwd}"
  export ADF_TEST_LOCAL_AUTH_DB="${adf_test_local_auth_db}"
  export ADF_TEST_LOCAL_HOST="${adf_test_local_host}"
  export MDB_TEST_LOCAL_PORT="${mdb_test_local_port}"
  export ADF_TEST_LOCAL_DB="${adf_test_local_db}"
  export ADF_TEST_URI="${adf_test_uri}"
  export SRV_TEST_DB="${srv_test_db}"
  export SRV_TEST_AUTH_DB="${srv_test_auth_db}"
  export SRV_TEST_HOST="${srv_test_host}"
  export SRV_TEST_USER="${srv_test_user}"
  export SRV_TEST_PWD="${srv_test_pwd}"
  export SCRIPT_FOLDER="$SCRIPT_FOLDER"
  export SCRIPT_DIR="$(pwd)/$SCRIPT_FOLDER"
  export SBOM_DIR="sbom_tools"
  export SBOM_LICENSES="mongo-odbc-driver.licenses.cdx.json"
  export SBOM_VULN="mongo-odbc-driver.merge.grype.cdx.json"
  export SBOM_FINAL="mongo-odbc-driver.full.cdx.json"
  export COMPLIANCE_REPORT_NAME="$COMPLIANCE_REPORT_NAME"
  export STATIC_CODE_ANALYSIS_NAME="$STATIC_CODE_ANALYSIS_NAME"
  export ALLOW_VULNS="${AllowVulns}"

  # Windows variables
  export LOCAL_DUMP_ORIGINAL_REG_VAL="$LOCAL_DUMP_ORIGINAL_REG_VAL"
  export DUMP_FOLDER="$DUMP_FOLDER"
  export DUMP_PATH="$(pwd)/$DUMP_FOLDER"
  export MONGOODBC_DEBUGGING_INFO_ARCHIVE=$MONGOODBC_DEBUGGING_INFO_ARCHIVE

  # Non-Windows variables
  export INSTALLED_ODBC_PATH="$INSTALLED_ODBC_PATH"
  export LD_LIBRARY_PATH="$LD_LIBRARY_PATH"
  export LIBRARY_PATH="$LD_LIBRARY_PATH"
  export ODBCSYSINI="$(pwd)"/setup

  # Common test infra variables
  export PROJECT_DIRECTORY="$PROJECT_DIRECTORY"
  export DRIVERS_TOOLS="$DRIVERS_TOOLS"
  export MONGO_ORCHESTRATION_HOME="$MONGO_ORCHESTRATION_HOME"
  export MONGODB_BINARIES="$MONGODB_BINARIES"
  export COMMON_TEST_INFRA_DIR="$COMMON_TEST_INFRA_DIR"
EOT

cat expansions.yml
