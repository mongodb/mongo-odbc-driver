use actix_web::rt;
use open;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, RedirectUrl, Scope,
};
use rfc8252_http_server::{threaded_start, OidcResponseParams};
use std::{collections::HashSet, hash::RandomState, time::Instant};

// temporary until rust driver OIDC support is released
// TODO Remove Me.
#[derive(Clone, Debug)]
pub struct IdpServerInfo {
    pub issuer: String,
    pub client_id: String,
    pub request_scopes: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CallbackContext {
    pub timeout_seconds: Option<Instant>,
    pub version: u32,
    pub refresh_token: Option<String>,
    pub idp_info: Option<IdpServerInfo>,
}

#[derive(Debug)]
pub struct IdpServerResponse {
    pub access_token: String,
    pub expires: Option<Instant>,
    pub refresh_token: Option<String>,
}
// END termporaries

#[derive(Debug)]
pub enum Error {
    IssuerUriMustBeHttps,
    NoIdpServerInfo,
    CsrfMismatch,
    Other(String),
}

pub async fn do_auth_flow(params: CallbackContext) -> Result<IdpServerResponse, Error> {
    let idp_info = params.idp_info.ok_or(Error::NoIdpServerInfo)?;
    let client_id = idp_info.client_id;
    let issuer_uri = IssuerUrl::new(idp_info.issuer).map_err(|e| Error::Other(e.to_string()))?;
    if issuer_uri.url().scheme() != "http" {
        return Err(Error::IssuerUriMustBeHttps);
    }
    let scopes = idp_info.request_scopes.unwrap_or_else(|| vec![]);

    let (server, oidc_params_channel) = threaded_start();

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_uri, async_http_client)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    // Create an OpenID Connect client by specifying the client ID, client secret,
    // authorization URL and token URL.
    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(client_id),
        None,
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:9080/callback".to_string())
            .map_err(|e| Error::Other(e.to_string()))?,
    );

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_url = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );

    let empty_vec = Vec::new();

    // Define the desired scopes based on the scopes passed in and the scopes available
    // on the server
    let scopes_supported: HashSet<String, RandomState> = HashSet::from_iter(
        provider_metadata
            .scopes_supported()
            .unwrap_or(&empty_vec)
            .into_iter()
            .map(|s| s.to_string()),
    );
    let desired_scopes = HashSet::from_iter(scopes.into_iter());
    // Set the desired scopes.
    for scope in desired_scopes.intersection(&scopes_supported) {
        // There does not seem to be a way to do intersection without cloning the scope
        auth_url = auth_url.add_scope(Scope::new(scope.clone()));
    }
    // Generate the full authorization URL.
    let (auth_url, csrf_token, _nonce) = auth_url
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    open::that(auth_url.to_string()).map_err(|e| Error::Other(e.to_string()))?;
    // awaiting on the listener waits for an actual response
    // the poc used a out-of-process proxy server that forwarded the code via GET,
    // but this in process server allows us to just await on the auth_code, and response_csrf.
    let OidcResponseParams { code, state } = oidc_params_channel
        .recv()
        .unwrap()
        .map_err(|e| Error::Other(e.to_string()))?;

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the
    // `response_csrf` (`state`) parameter returned by the server matches `csrf_token`.
    if let Some(state) = state {
        if state != *csrf_token.secret() {
            return Err(Error::CsrfMismatch);
        }
    }

    // Now you can exchange it for an access token and ID token.
    // implementation must implement RFC9207
    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    // Extract the auth and refresh tokens, and the expiration duration in seconds
    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires = token_response.expires_in();

    rt::System::new().block_on(server.stop(true));
    Ok(IdpServerResponse {
        access_token,
        expires: if let Some(expires) = expires {
            Some(Instant::now() + expires)
        } else {
            None
        },
        refresh_token,
    })
}
