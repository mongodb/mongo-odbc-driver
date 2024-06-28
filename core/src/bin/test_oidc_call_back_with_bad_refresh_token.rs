use mongo_odbc_core::{oidc_auth::*, test_config::*};
use mongodb::options::oidc::{CallbackContext, IdpServerInfo};

// This shows that the oidc_call_back function works. The second call will use the refresh token,
// which will be obvious in that the web browser only opens once.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let c = CallbackContext::builder()
        .idp_info(
            IdpServerInfo::builder()
                .issuer(ISSUER_URL.to_string())
                .client_id(Some(CLIENT_ID.to_string()))
                .request_scopes(Some(vec!["openid".to_string()]))
                .build(),
        )
        .refresh_token(Some(BAD_REFRESH_TOKEN.to_string()))
        .version(1u32)
        .build();
    // this should return something similar to: Some(Other("OpenID Connect: code exchange failed:
    // Server returned error response: StandardErrorResponse { error: invalid_grant,
    // error_description: Some(\"The refresh token is invalid or expired.\"), error_uri: None }"))
    println!(
        "This will result in an error: {:?}",
        oidc_call_back(c).await.unwrap_err().get_custom::<Error>()
    );
}
