use mongo_odbc_core::oidc_auth::*;

#[tokio::main]
async fn main() {
    let c = CallbackContext {
        idp_info: Some(IdpServerInfo {
            issuer: "https://dev-bzkxrnbykc6fb01i.us.auth0.com/".to_string(),
            client_id: "80OQwYGwA5JkCFnnQIdcITg3zlOjWfTO".to_string(),
            request_scopes: Some(vec!["openid".to_string(), "profile".to_string(), "offline_access".to_string()]),
        }),
        refresh_token: None,
        timeout_seconds: None,
        version: 1,
    };
    do_auth_flow(c).await.unwrap();
}
