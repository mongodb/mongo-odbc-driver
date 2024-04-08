use open;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, RedirectUrl, RequestTokenError, Scope,
};
use rfc8252_http_server::{start, OidcResponseParams};
use std::{collections::HashSet, hash::RandomState, time::Instant};

const DEFAULT_REDIRECT_URI: &str = "http://localhost:27097/redirect";

// temporary until rust driver OIDC support is released
// TODO SQL-1937: Remove Me.
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
    if issuer_uri.url().scheme() != "https" {
        return Err(Error::IssuerUriMustBeHttps);
    }
    let scopes = idp_info.request_scopes.unwrap_or_else(Vec::new);

    let (server, mut oidc_params_channel) = start().await;

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
        RedirectUrl::new(DEFAULT_REDIRECT_URI.to_string())
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
            .iter()
            .map(|s| s.to_string()),
    );
    let mut desired_scopes = HashSet::from_iter(scopes.into_iter());
    // mongodb is not configured to ask for offline_access by default. We prefer always getting a
    // refresh token when the server allows it.
    desired_scopes.insert("offline_access".to_string());

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
        .await
        .ok_or_else(|| Error::Other("No response from OIDC server".to_string()))?
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
    let token_request = client
        .exchange_code(AuthorizationCode::new(code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier);

    let token_response = token_request
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            let msg = match e {
                RequestTokenError::ServerResponse(provider_err) => {
                    format!("Server returned error response: {:?}", provider_err)
                }
                RequestTokenError::Request(req) => {
                    format!("Request failed: {:?}", req)
                }
                RequestTokenError::Parse(parse_err, res) => {
                    let body = match std::str::from_utf8(&res) {
                        Ok(text) => text.to_string(),
                        Err(_) => format!("{:?}", &res),
                    };
                    format!(
                        "Failed to parse server response: {} [response={:?}]",
                        parse_err, body
                    )
                }
                RequestTokenError::Other(msg) => msg,
            };
            Error::Other(format!("OpenID Connect: code exchange failed: {}", msg))
        })?;

    // Extract the auth and refresh tokens, and the expiration duration in seconds
    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires_in = token_response.expires_in();

    server.stop(true).await;

    Ok(IdpServerResponse {
        access_token,
        expires: expires_in.map(|e| Instant::now() + e),
        refresh_token,
    })
}
