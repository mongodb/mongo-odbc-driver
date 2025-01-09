mod common;

#[cfg(feature = "cluster_type_tests")]
mod cluster_type {
    use crate::common::{allocate_env, connect_with_conn_string};
    use definitions::AttrOdbcVersion;
    use std::panic;
    use tokio;

    use constants::DRIVER_NAME;
    use std::env;

    #[derive(Debug)]
    #[allow(dead_code)]
    enum PortType {
        Enterprise,
        Community,
    }

    fn connect_for_mdb_test(port_type: PortType) -> Result<(), String> {
        let connection_string = generate_mdb_connection_str(port_type);

        let env_handle = allocate_env(AttrOdbcVersion::SQL_OV_ODBC3);
        match connect_with_conn_string(env_handle, Some(connection_string), true) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn generate_mdb_connection_str(port_type: PortType) -> String {
        let user_name = env::var("LOCAL_MDB_USER").expect("LOCAL_MDB_USER is not set");
        let pwd = env::var("LOCAL_MDB_PWD").expect("LOCAL_MDB_PWD is not set");

        let port_var = match port_type {
            PortType::Enterprise => "LOCAL_MDB_PORT_ENT",
            PortType::Community => "LOCAL_MDB_PORT_COM",
        };
        let port = env::var(port_var).expect(&format!("{} is not set", port_var));

        let server = format!("localhost:{}", port);

        let db = env::var("ADF_TEST_LOCAL_DB");
        let driver = env::var("ADF_TEST_LOCAL_DRIVER").unwrap_or_else(|_| DRIVER_NAME.to_string());

        let mut connection_string =
            format!("Driver={{{driver}}};USER={user_name};PWD={pwd};SERVER={server};");

        if let Ok(val) = db {
            connection_string.push_str(&format!("DATABASE={};", val));
        }

        connection_string
    }

    async fn run_cluster_type_test(port_type: PortType) -> Result<(), String> {
        let result = panic::catch_unwind(|| connect_for_mdb_test(port_type));

        result.unwrap_or_else(|panic_info| {
            if let Some(s) = panic_info.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                Err(s.to_string())
            } else {
                Err("Unknown panic occurred".to_string())
            }
        })
    }

    // Tests that connection with community edition fails
    #[tokio::test]
    #[ignore = "SQL-2288: need real libmongosqltranslate"]
    async fn test_determine_cluster_type_community_fails() {
        let result = run_cluster_type_test(PortType::Community).await;
        assert!(
            result.is_err(),
            "Expected an error for community edition, but got success"
        );
        if let Err(e) = result {
            assert!(
                e.contains("Unsupported cluster configuration: Community edition detected") &&
                    e.contains("The driver is intended for use with MongoDB Enterprise edition or Atlas Data Federation"),
                "Unexpected error message for community edition: {}",
                e
            );
        }
    }

    // Tests that connection with enterprise edition and library loaded fails
    // due to missing 'sqlGetResultSchema' command in MongoDB
    #[tokio::test]
    #[ignore = "SQL-2288: need real libmongosqltranslate"]
    async fn test_enterprise_with_library_fails_due_to_missing_sql_get_result_schema_command() {
        let result = run_cluster_type_test(PortType::Enterprise).await;
        assert!(
            result.is_err(),
            "Expected an error with enterprise edition and library loaded, but got success"
        );
        if let Err(e) = result {
            assert!(
                e.contains("no such command: 'sqlGetResultSchema'"),
                "Unexpected error message for enterprise edition: {}",
                e
            );
        }
    }

    // Test that connecting with enterprise edition cluster type fails without mongosqltranslate library
    #[tokio::test]
    async fn test_determine_cluster_type_enterprise_fails_without_library() {
        let result = run_cluster_type_test(PortType::Enterprise).await;
        assert!(
            result.is_err(),
            "Expected an error for enterprise edition without mongosqltranslate library, but got success"
        );
        if let Err(e) = result {
            assert!(
                e.contains("Enterprise edition detected, but mongosqltranslate library not found."),
                "Unexpected error message for enterprise edition without library: {}",
                e
            );
        }
    }
}
