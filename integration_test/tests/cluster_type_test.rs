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
    async fn test_determine_cluster_type_community_fails() {
        let result = run_cluster_type_test(PortType::Community).await;
        assert!(
            result.is_err(),
            "Expected an error for community edition, but got success"
        );
        if let Err(e) = result {
            assert!(e.contains("Unsupported cluster configuration"));
            #[cfg(not(feature = "eap"))]
            assert!(
                e.contains("The driver is intended for use with MongoDB Atlas Data Federation"),
                "Unexpected error message for community edition: {}",
                e
            );
            #[cfg(feature = "eap")]
            assert!(
                    e.contains("The driver is intended for use with MongoDB Enterprise edition or Atlas Data Federation"),
                "Unexpected error message for community edition: {}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_determine_cluster_type_enterprise_succeeds() {
        let result = run_cluster_type_test(PortType::Enterprise).await;
        #[cfg(not(feature = "eap"))]
        assert!(
            result.is_err(),
            "Expected an error for enterprise edition, but got success"
        );
        #[cfg(feature = "eap")]
        assert!(
            result.is_ok(),
            "Expected success with enterprise edition, but got an error: {result:?}"
        );
    }
}
