use actix_web::{
    self, dev::ServerHandle, http, middleware, rt, web, App, HttpRequest, HttpResponse, HttpServer,
    Result,
};
use askama::Template;
use lazy_static::lazy_static;
use serde_json::Value;
use std::{sync::mpsc, thread, time};

// template page keys are:
// 'OIDCErrorPage'
// 'OIDCAcceptedPage'
// 'OIDCNotFoundPage'

lazy_static! {
    static ref CURRENT_DIR: String = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    static ref RESOURCES_DIR: String = format!("{}/rfc8252_http_server/resources", *CURRENT_DIR);
    static ref OIDC_SERVER_HTML_TEMPLATE: Value = serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/oidc_server_html_templates.json",
            *RESOURCES_DIR
        ))
        .unwrap()
    )
    .unwrap();
}

#[derive(Template)]
#[template(path = "OIDCAcceptedTemplate.html")]
struct OIDCAcceptedPage<'a> {
    product_docs_link: &'a str,
    product_docs_name: &'a str,
    error: &'a str,
    error_uri: &'a str,
    error_description: &'a str,
}

async fn index(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(
            OIDCAcceptedPage {
                product_docs_link: "https://www.example.com",
                product_docs_name: "Example",
                error: "error",
                error_uri: "error_uri",
                error_description: "error_description",
            }
            .render()
            .unwrap(),
        ))
}

async fn run_app(sender: mpsc::Sender<ServerHandle>) -> std::io::Result<()> {
    println!("starting HTTP server at http://localhost:9080");

    // srv is server controller type, `dev::Server`
    let server = HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
    })
    .bind(("127.0.0.1", 9080))?
    .workers(2)
    .run();

    // Send server handle back to the main thread
    let _ = sender.send(server.handle());

    server.await
}

pub fn start() {
    let (sender, receiver) = mpsc::channel();

    println!("spawning thread for server");
    thread::spawn(move || {
        let server_future = run_app(sender);
        rt::System::new().block_on(server_future)
    });

    let server_handle = receiver.recv().unwrap();

    println!("waiting 10 seconds");
    thread::sleep(time::Duration::from_secs(10));

    // Send a stop signal to the server, waiting for it to exit gracefully
    println!("stopping server");
    rt::System::new().block_on(server_handle.stop(true));
}

pub struct RFC8252HttpServerOptions {
    pub redirect_uri: String,
    pub oidc_state_param: String,
}

pub struct RFC8252HttpServer {
    options: RFC8252HttpServerOptions,
}
