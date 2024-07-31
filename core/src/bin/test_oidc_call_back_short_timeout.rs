use mongo_odbc_core::{
    oidc_auth::{oidc_call_back, Error},
    test_config::*,
};
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
        .timeout(std::time::Instant::now() + std::time::Duration::from_secs(2))
        .version(1u32)
        .build();
    println!("This is very timing dependent, but we should see a timeout, if we play with the value: {:?}", oidc_call_back(c).await.unwrap_err().get_custom::<Error>());
}
