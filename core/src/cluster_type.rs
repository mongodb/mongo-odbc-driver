use crate::{err::Result, Error};
use mongodb::bson::{doc, Bson, Document};
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum MongoClusterType {
    AtlasDataFederation,
    Community,
    Enterprise,
    UnknownTarget,
}

pub async fn determine_cluster_type(client: &Client) -> Result<MongoClusterType> {
    let db = client.database("admin");

    // The { buildInfo: 1 } command returns information that indicates
    // the type of the cluster.
    let build_info_cmd = doc! { "buildInfo": 1 };
    let cmd_res: Document = db
        .run_command(build_info_cmd)
        .await
        .map_err(Error::BuildInfoCmdExecutionFailed)?;

    // if "ok" is not 1, then the target type could not be determined.
    match cmd_res.get("ok") {
        Some(Bson::Double(f)) if *f == 1.0 => {}
        Some(Bson::Int64(i)) if *i == 1 => {}
        Some(Bson::Int32(i)) if *i == 1 => {}
        _ => {
            log::error!("buildInfo command returned a non-ok response: {cmd_res:?}");
            return Ok(MongoClusterType::UnknownTarget);
        }
    }

    // If the "dataLake" field is present, it must be an ADF cluster.
    if cmd_res.get_document("dataLake").is_ok() {
        Ok(MongoClusterType::AtlasDataFederation)
    } else {
        // Otherwise, if "modules" is present and contains "enterprise",
        // this must be an Enterprise cluster.
        match cmd_res.get_array("modules") {
            Ok(modules) => {
                if modules
                    .iter()
                    .any(|mod_name| mod_name.as_str() == Some("enterprise"))
                {
                    Ok(MongoClusterType::Enterprise)
                } else {
                    Ok(MongoClusterType::Community)
                }
            }
            Err(_) => Ok(MongoClusterType::Community),
        }
    }
}
