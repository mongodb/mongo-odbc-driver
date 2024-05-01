use mongo_odbc_core::oidc_auth::oidc_call_back;
use mongodb::options::oidc::{CallbackContext, IdpServerInfo};

// This shows that the oidc_call_back function works. The second call will use the refresh token,
// which will be obvious in that the web browser only opens once.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let c = CallbackContext::builder()
        .idp_info(
            IdpServerInfo::builder()
                .issuer("https://mongodb-dev.okta.com/oauth2/ausqrxbcr53xakaRR357".to_string())
                .client_id(Some("0oarvap2r7PmNIBsS357".to_string()))
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
