use mongo_odbc_core::oidc_auth::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let c = CallbackContext {
        idp_info: Some(IdpServerInfo {
            issuer: "https://mongodb-dev.okta.com/oauth2/ausqrxbcr53xakaRR357".to_string(),
            client_id: Some("0oarvap2r7PmNIBsS357".to_string()),
            request_scopes: Some(vec!["openid".to_string()]),
        }),
        refresh_token: Some("PnWePzgf-4BqC048C5Q7BhtBxfu3nId9e72y5T5-PLj".to_string()),
        // 2 seconds works well on my laptop. 1 second results in the request dying before the
        //   server can redirect
        timeout_seconds: Some(std::time::Instant::now() + std::time::Duration::from_secs(2)),
        version: 1,
    };
    // this should return something similar to: Some(Other("OpenID Connect: code exchange failed:
    // Server returned error response: StandardErrorResponse { error: invalid_grant,
    // error_description: Some(\"The refresh token is invalid or expired.\"), error_uri: None }"))
    println!(
        "This will result in an error: {:?}",
        oidc_call_back(c).await.unwrap_err().get_custom::<Error>()
    );
}
