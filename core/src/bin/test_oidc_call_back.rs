use mongo_odbc_core::{oidc_auth::oidc_call_back, test_config::*};
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
        .version(1u32)
        .build();
    let mut refresh_c = c.clone();
    let res = oidc_call_back(c).await.unwrap();
    println!("initial refresh token: {:?}", res.refresh_token);
    refresh_c.refresh_token = res.refresh_token;
    println!(
        "second response: {:?}",
        oidc_call_back(refresh_c).await.unwrap()
    );
}
