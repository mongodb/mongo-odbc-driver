use mongodb::bson::{doc, Document};
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum MongoClusterType {
    AtlasDataFederation,
    Community,
    Enterprise,
    UnknownTarget,
}

pub async fn determine_cluster_type(client: &Client) -> MongoClusterType {
    let db = client.database("admin");
    let build_info_cmd = doc! { "buildInfo": 1 };

    // Run the command and handle any errors internally
    let cmd_res: Document = match db.run_command(build_info_cmd).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to run buildInfo command: {:?}", e);
            return MongoClusterType::UnknownTarget;
        }
    };

    // Check if the "ok" field is 1
    if cmd_res.get_f64("ok").unwrap_or(0.0) != 1.0 {
        log::error!(
            "buildInfo command returned a non-ok response: {:?}",
            cmd_res
        );
        return MongoClusterType::UnknownTarget;
    }

    // Determine the cluster type based on the response
    if cmd_res.get_document("dataLake").is_ok() {
        MongoClusterType::AtlasDataFederation
    } else {
        match cmd_res.get_array("modules") {
            Ok(modules) => {
                if modules
                    .iter()
                    .any(|mod_name| mod_name.as_str() == Some("enterprise"))
                {
                    MongoClusterType::Enterprise
                } else {
                    MongoClusterType::Community
                }
            }
            Err(_) => MongoClusterType::Community,
        }
    }
}
