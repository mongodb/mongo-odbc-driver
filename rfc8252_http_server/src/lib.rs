use actix_web::{
    self, dev::ServerHandle, http, middleware, rt, web, App, HttpRequest, HttpResponse, HttpServer,
    Result,
};
use askama::Template;
use std::{collections::HashMap, sync::mpsc, thread};
use tokio::sync::mpsc as tokio_mpsc;

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

#[derive(Debug, Clone)]
pub struct OidcResponseParams {
    pub code: String,
    pub state: Option<String>,
}

async fn threaded_callback(
    oidc_params_sender: mpsc::Sender<OidcResponseParams>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let query = req.query_string();
    let params_vec: Vec<_> = query.split('&').collect();
    if params_vec.is_empty() || params_vec.get(0).unwrap().is_empty() {
        return threaded_error(
            oidc_params_sender,
            "unknown error",
            "response parameters are missing",
        )
        .await;
    }
    let params = params_vec
        .into_iter()
        .map(|kv| {
            let kv: Vec<_> = kv.split('=').collect();
            match kv.len() {
                0 => ("", ""),
                1 => (kv[0], ""),
                _ => (kv[0], kv[1]),
            }
        })
        .collect::<HashMap<&str, &str>>();

    let code = params.get("code");
    if let Some(code) = code {
        let state = params.get("state").map(|s| s.to_string());
        let oidc_response_params = OidcResponseParams {
            code: code.to_string(),
            state,
        };
        let _ = oidc_params_sender.send(oidc_response_params);
        threaded_accepted().await
    } else if let Some(e) = params.get("error") {
        if let Some(error_description) = params.get("error_description") {
            threaded_error(oidc_params_sender, e, error_description).await
        } else {
            threaded_error(oidc_params_sender, e, "no error description was provided").await
        }
    } else {
        threaded_error(
            oidc_params_sender,
            "unknown error",
            "response parameters are missing required information",
        )
        .await
    }
}

async fn threaded_stop(stop_sender: mpsc::Sender<bool>) -> Result<HttpResponse> {
    let _ = stop_sender.send(true);
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("<html><body>Server is stopping</body></html>".to_string()))
}

async fn threaded_accepted() -> Result<HttpResponse> {
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

async fn threaded_error(
    oidc_params_sender: mpsc::Sender<OidcResponseParams>,
    error: &str,
    error_description: &str,
) -> Result<HttpResponse> {
    let _ = oidc_params_sender.send(OidcResponseParams {
        code: "".to_string(),
        state: None,
    });
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(
            OIDCErrorPage {
                product_docs_link: "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect",
                product_docs_name: "Atlas SQL ODBC Driver",
                error,
                error_uri: "https://www.mongodb.com/docs/atlas/data-federation/query/sql/drivers/odbc/connect/bad_oidc_login",
                error_description,
            }
            .render()
            .unwrap(),
        ))
}

async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
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

async fn threaded_run_app(
    sender: mpsc::Sender<ServerHandle>,
    oidc_params_sender: mpsc::Sender<OidcResponseParams>,
    stop_sender: mpsc::Sender<bool>,
) -> std::io::Result<()> {
    println!("starting HTTP server at http://localhost:9080");

    // srv is server controller type, `dev::Server`
    let server = HttpServer::new(move || {
        let stop_sender = stop_sender.clone();
        let oidc_params_sender = oidc_params_sender.clone();
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/callback")
                    .to(move |r| threaded_callback(oidc_params_sender.clone(), r)),
            )
            .service(web::resource("/stop").to(move || threaded_stop(stop_sender.clone())))
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", 9080))?
    .workers(2)
    .run();

    // Send server handle back to the main thread
    let _ = sender.send(server.handle());

    server.await
}

pub fn threaded_start() -> std::result::Result<OidcResponseParams, std::io::Error> {
    let (sender, receiver) = mpsc::channel();
    let (oidc_params_sender, oidc_params_receiver) = mpsc::channel();
    let (stop_sender, stop_receiver) = mpsc::channel();

    println!("spawning thread for server");
    thread::spawn(move || {
        let server_future = threaded_run_app(sender, oidc_params_sender, stop_sender);
        rt::System::new().block_on(server_future)
    });

    let server_handle = receiver.recv().unwrap();

    loop {
        if let Ok(oidc_params) = oidc_params_receiver.try_recv() {
            // Send a stop signal to the server, waiting for it to exit gracefully
            println!("stopping server");
            rt::System::new().block_on(server_handle.stop(true));
            return Ok(oidc_params);
        }
        if let Ok(_) = stop_receiver.try_recv() {
            println!("received stop signal");
            // Send a stop signal to the server, waiting for it to exit gracefully
            println!("stopping server");
            rt::System::new().block_on(server_handle.stop(true));
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "server was stopped",
            ));
        }
    }
}
