use mongodb::options::oidc::{CallbackContext, IdpServerResponse};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    http, AsyncHttpClient, AuthorizationCode, ClientId, ConfigurationError, CsrfToken,
    HttpClientError, HttpResponse, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    RedirectUrl, RefreshToken, RequestTokenError, Scope,
};
use reqwest::{redirect, Client};
use rfc8252_http_server::{start, OidcResponseParams};
use std::{collections::HashSet, future::Future, pin::Pin, time::Instant};
use tokio::time::{self, Duration};

const DEFAULT_REDIRECT_URI: &str = "http://localhost:27097/redirect";
const DEFAULT_SLEEP_DURATION: Duration = Duration::from_secs(5 * 60); // from_mins is unstable, so we use from_secs with a multiplication. The multiplication is performed at compile time, anyway
const OFFLINE_ACCESS_SCOPE: &str = "offline_access";
const OPENID_SCOPE: &str = "openid";

#[derive(Debug)]
pub enum Error {
    ClientBuild(reqwest::Error),
    CodeExchange(ConfigurationError),
    IssuerUriMustBeHttps(String),
    NoIdpServerInfo,
    NoRefreshToken,
    CsrfMismatch,
    HumanFlowUnsupported,
    Timedout,
    Other(String),
}

impl From<Error> for mongodb::error::Error {
    fn from(e: Error) -> Self {
        mongodb::error::Error::custom(e)
    }
}

/// Creates an async HTTP client backed by reqwest for openidconnect.
fn async_http_client() -> Result<ReqwestAsyncClient, reqwest::Error> {
    // HTTPS client for upstream requests
    //
    // Note: Redirects should _not_ be enabled as they are susceptible to SSRF vulnerabilities
    let client = Client::builder()
        .redirect(redirect::Policy::none())
        .build()?;
    Ok(ReqwestAsyncClient(client))
}

/// Wrapper around a reqwest client for use with openidconnect.
///
/// Note: This is shamelessly copied from the `oauth2-reqwest` crate since that
/// crate is neither stable nor will fix the dependency on an outdated version
/// of reqwest.
///
/// Original source: <https://github.com/ramosbugs/oauth2-rs/blob/72ce74401c26eb4dc85dcbfde587bbcfc149e3ae/oauth2-reqwest/src/lib.rs#L119>
/// Original license: MIT
#[derive(Debug, Clone)]
struct ReqwestAsyncClient(Client);

impl<'c> AsyncHttpClient<'c> for ReqwestAsyncClient {
    type Error = HttpClientError<reqwest::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + Sync + 'c>>;

    fn call(&'c self, request: openidconnect::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self
                .0
                .execute(request.try_into().map_err(Box::new)?)
                .await
                .map_err(Box::new)?;

            let mut builder = http::Response::builder().status(response.status());
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.version(response.version());
            }
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            builder
                .body(response.bytes().await.map_err(Box::new)?.to_vec())
                .map_err(HttpClientError::Http)
        })
    }
}

/// Executes the OIDC callback, choosing between refresh and full auth flow.
///
/// # Errors
/// Returns [`mongodb::error::Error`] if the flow times out or the underlying
/// [`do_refresh`] or [`do_auth_flow`] call fails.
pub async fn oidc_call_back(params: CallbackContext) -> mongodb::error::Result<IdpServerResponse> {
    let sleep_duration = params
        .timeout
        // turn the supplied timeout Instant into a Duration from now
        .map_or(DEFAULT_SLEEP_DURATION, |x| x - Instant::now());

    // If there is a refresh token, we refresh, otherwise we do not
    if params.refresh_token.is_some() {
        Ok(time::timeout(sleep_duration, do_refresh(params))
            .await
            .map_err(|_| Error::Timedout)??)
    } else {
        Ok(time::timeout(sleep_duration, do_auth_flow(params))
            .await
            .map_err(|_| Error::Timedout)??)
    }
}

pub(crate) fn build_scopes(
    idp_info: &mongodb::options::oidc::IdpServerInfo,
    provider_metadata: &CoreProviderMetadata,
) -> impl Iterator<Item = Scope> {
    let mut requested_scopes = idp_info.request_scopes.clone().unwrap_or_default();
    // We always want to request OFFLINE_ACCESS, if supported by the IdP.
    requested_scopes.push(OFFLINE_ACCESS_SCOPE.to_string());
    // Always include the openid scope, even if it is not claimed by the IdP.
    let mut scopes = vec![OPENID_SCOPE.to_string()];
    let supported_scopes = provider_metadata
        .scopes_supported()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();
    if let Some(client_id) = &idp_info.client_id {
        // If the client_id is provided, we add the default scope for it.
        // This is necessary for Azure OIDC, which uses a special scope format.
        let client_id_default = format!("{client_id}/.default");
        if requested_scopes.contains(&client_id_default) {
            scopes.push(client_id_default);
        }
    }
    // If supported scopes is empty, we just assume reporting is not correct and attempt with what
    // is requested.
    if supported_scopes.is_empty() {
        scopes.extend(requested_scopes);
    } else {
        for scope in requested_scopes {
            if supported_scopes.contains(&scope) {
                scopes.push(scope);
            } else {
                log::warn!(
                    "Requested scope '{scope}' is not supported by the OIDC provider, skipping.",
                );
            }
        }
    }
    scopes.into_iter().map(Scope::new)
}

pub async fn do_auth_flow(params: CallbackContext) -> Result<IdpServerResponse, Error> {
    let idp_info = params.idp_info.ok_or(Error::NoIdpServerInfo)?;
    let client_id = idp_info
        .client_id
        .clone()
        .ok_or(Error::HumanFlowUnsupported)?;
    let issuer_uri =
        IssuerUrl::new(idp_info.issuer.clone()).map_err(|e| Error::Other(e.to_string()))?;
    if issuer_uri.url().scheme() != "https" {
        return Err(Error::IssuerUriMustBeHttps(
            issuer_uri.url().scheme().to_string(),
        ));
    }
    let async_http_client = async_http_client().map_err(Error::ClientBuild)?;
    let (server, mut oidc_params_channel) = start().await;

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_uri, &async_http_client)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    // Create an OpenID Connect client by specifying the client ID, client secret,
    // authorization URL and token URL.
    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(client_id.clone()),
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

    for scope in build_scopes(&idp_info, &provider_metadata) {
        // There does not seem to be a way to do intersection without cloning the scope
        auth_url = auth_url.add_scope(scope);
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
        .map_err(Error::CodeExchange)?
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier);

    let token_response = token_request
        .request_async(&async_http_client)
        .await
        .map_err(|e| {
            let msg = match e {
                RequestTokenError::ServerResponse(provider_err) => {
                    format!("Server returned error response: {provider_err:?}")
                }
                RequestTokenError::Request(req) => {
                    format!("Request failed: {req:?}")
                }
                RequestTokenError::Parse(parse_err, res) => {
                    let body = match std::str::from_utf8(&res) {
                        Ok(text) => text.to_string(),
                        Err(_) => format!("{:?}", &res),
                    };
                    format!("Failed to parse server response: {parse_err} [response={body:?}]")
                }
                RequestTokenError::Other(msg) => msg,
            };
            Error::Other(format!("OpenID Connect: code exchange failed: {msg}"))
        })?;

    // Extract the auth and refresh tokens, and the expiration duration in seconds
    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires = token_response.expires_in();

    server.stop(true).await;

    Ok(IdpServerResponse::builder()
        .access_token(access_token)
        .expires(expires.map(|e| Instant::now() + e))
        .refresh_token(refresh_token)
        .build())
}

pub async fn do_refresh(params: CallbackContext) -> Result<IdpServerResponse, Error> {
    let idp_info = params.idp_info.ok_or(Error::NoIdpServerInfo)?;
    let client_id = idp_info.client_id.ok_or(Error::HumanFlowUnsupported)?;
    let issuer_uri = IssuerUrl::new(idp_info.issuer).map_err(|e| Error::Other(e.to_string()))?;
    if issuer_uri.url().scheme() != "https" {
        return Err(Error::IssuerUriMustBeHttps(
            issuer_uri.url().scheme().to_string(),
        ));
    }
    let async_http_client = async_http_client().map_err(Error::ClientBuild)?;

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_uri, &async_http_client)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    // Create an OpenID Connect client by specifying the client ID, client secret,
    // authorization URL and token URL.
    let client = CoreClient::from_provider_metadata(
        provider_metadata.clone(),
        ClientId::new(client_id),
        None,
    );

    // This function will never be called without a refresh token (to be checked in the driver function),
    // but we return an error to be explicit about the fact that we expect a refresh token.
    let token_response = client
        .exchange_refresh_token(&RefreshToken::new(
            params.refresh_token.ok_or(Error::NoRefreshToken)?,
        ))
        .map_err(Error::CodeExchange)?
        .request_async(&async_http_client)
        .await
        .map_err(|e| {
            let msg = match e {
                RequestTokenError::ServerResponse(provider_err) => {
                    format!("Server returned error response: {provider_err:?}")
                }
                RequestTokenError::Request(req) => {
                    format!("Request failed: {req:?}")
                }
                RequestTokenError::Parse(parse_err, res) => {
                    let body = match std::str::from_utf8(&res) {
                        Ok(text) => text.to_string(),
                        Err(_) => format!("{:?}", &res),
                    };
                    format!("Failed to parse server response: {parse_err} [response={body:?}]")
                }
                RequestTokenError::Other(msg) => msg,
            };
            Error::Other(format!("OpenID Connect: code exchange failed: {msg}"))
        })?;

    // Extract the auth and refresh tokens, and the expiration duration in seconds
    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires = token_response.expires_in();

    Ok(IdpServerResponse::builder()
        .access_token(access_token)
        .expires(expires.map(|e| Instant::now() + e))
        .refresh_token(refresh_token)
        .build())
}
