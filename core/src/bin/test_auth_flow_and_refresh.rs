use mongo_odbc_core::{
    oidc_auth::{do_auth_flow, do_refresh},
    test_config::*,
};
use mongodb::options::oidc::{CallbackContext, IdpServerInfo};

#[tokio::main]
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
    let res = do_auth_flow(c).await.unwrap();
    println!(
        "{:?}, {:?}, {:?}",
        res.access_token, res.expires, res.refresh_token
    );
    refresh_c.refresh_token = res.refresh_token;
    println!("{:?}", do_refresh(refresh_c).await.unwrap());
}
