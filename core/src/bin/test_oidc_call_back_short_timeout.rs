use mongo_odbc_core::oidc_auth::*;

#[tokio::main]
async fn main() {
    let c = CallbackContext {
        idp_info: Some(IdpServerInfo {
            issuer: "https://mongodb-dev.okta.com/oauth2/ausqrxbcr53xakaRR357".to_string(),
            client_id: Some("0oarvap2r7PmNIBsS357".to_string()),
            // show that we get a refresh_token even when we don't ask for it here
            request_scopes: Some(vec!["openid".to_string()]),
        }),
        refresh_token: None,
        // 2 seconds works well on my laptop. 1 second results in the request dying before the
        //   server can redirect
        timeout_seconds: Some(std::time::Instant::now() + std::time::Duration::from_secs(2)),
        version: 1,
    };
    println!("This is very timing dependent, but we should see a timeout, if we play with the value: {:?}", oidc_call_back(c).await.unwrap_err().get_custom::<Error>());
    // example output from my laptop: `This is very timing dependent, but we should see a timeout, if we play with the value: Some(Timedout)`
}
