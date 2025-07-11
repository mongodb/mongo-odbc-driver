use rfc8252_http_server::start;

// This is only for testing the appearance of the web pages
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (server, mut oidc_params) = start().await;
    let res = oidc_params.recv().await.unwrap();
    server.stop(true).await;
    println!("server result: {res:?}");
}
