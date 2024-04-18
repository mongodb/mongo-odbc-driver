use mongo_odbc_core::oidc_auth::*;

// This shows that the oidc_call_back function works. The second call will use the refresh token,
// which will be obvious in that the webbrower only opens once.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let c = CallbackContext {
        idp_info: Some(IdpServerInfo {
            issuer: "https://mongodb-dev.okta.com/oauth2/ausqrxbcr53xakaRR357".to_string(),
            client_id: Some("0oarvap2r7PmNIBsS357".to_string()),
            // show that we get a refresh_token even when we don't ask for it here
            request_scopes: Some(vec!["openid".to_string()]),
        }),
        refresh_token: None,
        timeout_seconds: None,
        version: 1,
    };
    let mut refresh_c = c.clone();
    let IdpServerResponse {
        access_token: _,
        expires: _,
        refresh_token,
    } = oidc_call_back(c).await.unwrap();
    println!("initial refresh token: {refresh_token:?}");
    refresh_c.refresh_token = refresh_token;
    println!(
        "second response: {:?}",
        oidc_call_back(refresh_c).await.unwrap()
    );
}
