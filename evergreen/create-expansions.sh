export CARGO_NET_GIT_FETCH_WITH_CLI=true
export INSTALLED_ODBC_PATH="$PWD/installed_odbc/install"
export LD_LIBRARY_PATH="$INSTALLED_ODBC_PATH/lib"
if [ "Windows_NT" == "$OS" ]; then
    export PATH="$HOME/.rustup/bin:$HOME/.cargo/bin:$LD_LIBRARY_PATH:$PATH"
else
    export PATH="$HOME/.cargo/bin:$LD_LIBRARY_PATH:$PATH"
fi
export DUMP_FOLDER=dumps
export LOCAL_DUMP_ORIGINAL_REG_VAL=local_dump_original_value.reg
export MONGOODBC_DEBUGGING_INFO_ARCHIVE=crashDebuggingInfo
export SCRIPT_FOLDER=resources
export COMPLIANCE_REPORT_NAME="mongo-odbc-driver_compliance_report.md"
export STATIC_CODE_ANALYSIS_NAME="mongo-odbc-driver.sast.sarif"
export FEATURE_FLAGS=""
export PRODUCT_NAME="mongoodbc"

echo "snapshot-eap: ${snapshot-eap}"

if [[ "${triggered_by_git_tag}" != "" ]]; then
    export RELEASE_VERSION=$(echo ${triggered_by_git_tag} | sed s/v//)

    # Check if this is a beta tag or snapshot-eap is set to true
    if [[ "${triggered_by_git_tag}" == *"beta"* || "${snapshot-eap}" == "true" ]]; then
        export FEATURE_FLAGS="eap"
        export PRODUCT_NAME="mongoodbc-eap"
    fi
else
    # If not a tag, we are in a snapshot build. We need to see if we're in beta mode or not
    # and set the release version to either snapshot or snapshot-eap
    if [[ "${snapshot-eap}" == "true" ]]; then
        export RELEASE_VERSION="snapshot-eap"
        export FEATURE_FLAGS="eap"
        export PRODUCT_NAME="mongoodbc-eap"
        echo "Building EAP version"
    else
        export RELEASE_VERSION="snapshot"
    fi
fi

export MSI_FILENAME="$PRODUCT_NAME-$RELEASE_VERSION.msi"
export UBUNTU_FILENAME="$PRODUCT_NAME-$RELEASE_VERSION.tar.gz"

cat <<EOT >expansions.yml
RELEASE_VERSION: "$RELEASE_VERSION"
FEATURE_FLAGS: "$FEATURE_FLAGS"
PRODUCT_NAME: "$PRODUCT_NAME"
MSI_FILENAME: "$MSI_FILENAME"
UBUNTU_FILENAME: "$UBUNTU_FILENAME"
WINDOWS_INSTALLER_PATH: "${PRODUCT_NAME}/mongosql-odbc-driver/windows/$RELEASE_VERSION/release/$MSI_FILENAME"
UBUNTU2204_INSTALLER_PATH: "${PRODUCT_NAME}/mongosql-odbc-driver/ubuntu2204/$RELEASE_VERSION/release/$UBUNTU_FILENAME"
COMPLIANCE_REPORT_NAME: "$COMPLIANCE_REPORT_NAME"
STATIC_CODE_ANALYSIS_NAME: "$STATIC_CODE_ANALYSIS_NAME"
prepare_shell: |
  set -o errexit
  export RELEASE_VERSION="$RELEASE_VERSION"
  export FEATURE_FLAGS="$FEATURE_FLAGS"
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
EOT

cat expansions.yml
