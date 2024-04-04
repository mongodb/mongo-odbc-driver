use actix_web::{
    self, dev::ServerHandle, http, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use askama::Template;
use std::collections::HashMap;
use std::result::Result as StdResult;
use tokio::sync::mpsc;

const DEFAULT_REDIRECT_PORT: u16 = 27097;
#[cfg(test)]
const DEFAULT_REDIRECT_URI: &str = "http://localhost:27097/redirect";

#[derive(Template)]
#[template(path = "OIDCAcceptedTemplate.html")]
struct OIDCAcceptedPage<'a> {
    product_docs_link: &'a str,
    product_docs_name: &'a str,
    error: &'a str,
    error_uri: &'a str,
    error_description: &'a str,
}

#[derive(Template)]
#[template(path = "OIDCErrorTemplate.html")]
struct OIDCErrorPage<'a> {
    product_docs_link: &'a str,
    product_docs_name: &'a str,
    error: &'a str,
    error_uri: &'a str,
    error_description: &'a str,
}

#[derive(Template)]
#[template(path = "OIDCNotFoundTemplate.html")]
struct OIDCNotFoundPage<'a> {
    product_docs_link: &'a str,
    product_docs_name: &'a str,
}

#[derive(Debug, Clone)]
pub struct RFC8252HttpServerOptions {
    pub redirect_uri: String,
    pub oidc_state_param: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcResponseParams {
    pub code: String,
    pub state: Option<String>,
}

// get the parameters from the query string as a HashMap
async fn get_params(query: &str) -> Result<HashMap<&str, &str>> {
    let params_vec: Vec<_> = query.split('&').collect();
    if params_vec.is_empty() || params_vec.first().unwrap().is_empty() {
        Err(actix_web::error::ErrorBadRequest(
            "response parameters are missing",
        ))?;
    }
    Ok(params_vec
        .into_iter()
        .map(|kv| {
            let kv: Vec<_> = kv.split('=').collect();
            match kv.len() {
                0 => ("", ""),
                1 => (kv[0], ""),
                _ => (kv[0], kv[1]),
            }
        })
        .collect::<HashMap<&str, &str>>())
}

// Implement the callback url action
async fn callback(
    oidc_params_sender: mpsc::Sender<StdResult<OidcResponseParams, String>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let query = req.query_string();
    let params = match get_params(query).await {
        Ok(params) => params,
        Err(e) => {
            return error(oidc_params_sender, "unknown error", &format!("{}", e)).await;
        }
    };

    let code = params.get("code");
    if let Some(code) = code {
        let state = params.get("state").map(|s| s.to_string());
        let oidc_response_params = OidcResponseParams {
            code: code.to_string(),
            state,
        };
        let _ = oidc_params_sender.send(Ok(oidc_response_params)).await;
        accepted().await
    } else if let Some(e) = params.get("error") {
        if let Some(error_description) = params.get("error_description") {
            error(oidc_params_sender, e, error_description).await
        } else {
            error(oidc_params_sender, e, "no error description was provided").await
        }
    } else {
        error(
            oidc_params_sender,
            "unknown error",
            "response parameters are missing required information",
        )
        .await
    }
}

// Implement the accepted page
async fn accepted() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(
            OIDCAcceptedPage {
                product_docs_link:
                    "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect",
                product_docs_name: "Atlas SQL ODBC Driver",
                error: "error",
                error_uri: "error_uri",
                error_description: "error_description",
            }
            .render()
            .unwrap(),
        ))
}

// Implement the error page and send the error on OIDC params channel
async fn error(
    oidc_params_sender: mpsc::Sender<StdResult<OidcResponseParams, String>>,
    error: &str,
    error_description: &str,
) -> Result<HttpResponse> {
    let _ = oidc_params_sender
        .send(Err(format!("{}: {}", error, error_description)))
        .await;
    Ok(HttpResponse::build(http::StatusCode::BAD_REQUEST)
        .content_type("text/html; charset=utf-8")
        .body(
            OIDCErrorPage {
                product_docs_link: "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect",
                product_docs_name: "Atlas SQL ODBC Driver",
                error,
                // TODO SQL-2008: make sure this page exists and possibly update the link if the
                // docs team has a preference
                error_uri: "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect/oidc_login_error",
                error_description,
            }
            .render()
            .unwrap(),
        ))
}

// Implement the not found page (404)
async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(
            OIDCNotFoundPage {
                product_docs_link: "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect",
                product_docs_name: "Atlas SQL ODBC Driver",
            }
            .render()
            .unwrap(),
        ))
}

// The main runner for the server
async fn run_app(
    sender: mpsc::Sender<ServerHandle>,
    oidc_params_sender: mpsc::Sender<StdResult<OidcResponseParams, String>>,
) -> std::io::Result<()> {
    // srv is server controller type, `dev::Server`
    let server = HttpServer::new(move || {
        let oidc_params_sender1 = oidc_params_sender.clone();
        let oidc_params_sender2 = oidc_params_sender.clone();
        App::new()
            .service(
                web::resource("/callback").to(move |r| callback(oidc_params_sender1.clone(), r)),
            )
            .service(
                web::resource("/redirect").to(move |r| callback(oidc_params_sender2.clone(), r)),
            )
            .default_service(web::route().to(not_found))
    })
    .bind(("localhost", DEFAULT_REDIRECT_PORT))?
    .workers(1)
    .run();

    // Send server handle back to the main thread
    let _ = sender.send(server.handle()).await;

    server.await
}

// The start function runs the main server runner in a tokio task and returns the server handle and
// a receiver channel for the OIDC response parameters/errors
pub async fn start() -> (
    ServerHandle,
    mpsc::Receiver<StdResult<OidcResponseParams, String>>,
) {
    let (sender, mut receiver) = mpsc::channel(1);
    let (oidc_params_sender, oidc_params_receiver) = mpsc::channel(1);

    tokio::spawn(async move {
        let server_future = run_app(sender, oidc_params_sender);
        server_future.await
    });

    let server_handle = receiver.recv().await.unwrap();

    (server_handle, oidc_params_receiver)
}

#[tokio::test]
async fn rfc8252_http_server_accepted() {
    use reqwest;
    let (server_handle, mut oidc_params_receiver) = start().await;
    let _ = reqwest::get(format!(
        "{}{}",
        DEFAULT_REDIRECT_URI, "?code=1234&state=foo"
    ))
    .await
    .unwrap();
    let oidc_params = oidc_params_receiver.recv().await.unwrap().unwrap();
    server_handle.stop(true).await;
    assert_eq!(oidc_params.code, "1234");
    assert_eq!(oidc_params.state, Some("foo".to_string()));
}

#[tokio::test]
async fn rfc8252_http_server_error() {
    let (server_handle, mut oidc_params_receiver) = start().await;
    let _ = reqwest::get(format!(
        "{}{}",
        DEFAULT_REDIRECT_URI, "?error=1234&error_description=foo"
    ))
    .await
    .unwrap();
    let oidc_params = oidc_params_receiver.recv().await.unwrap();
    server_handle.stop(true).await;
    assert_eq!(oidc_params, Err("1234: foo".to_string()));
}

#[tokio::test]
async fn rfc8252_http_server_no_params() {
    let (server_handle, mut oidc_params_receiver) = start().await;
    let _ = reqwest::get(DEFAULT_REDIRECT_URI).await.unwrap();
    let oidc_params = oidc_params_receiver.recv().await.unwrap();
    server_handle.stop(true).await;
    assert_eq!(
        oidc_params,
        Err("unknown error: response parameters are missing".to_string())
    );
}
