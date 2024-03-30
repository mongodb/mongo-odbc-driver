use rfc8252_http_server::tokio_start;

#[tokio::main]
async fn main() {
    let (server, mut oidc_params) = tokio_start().await;
    let res = oidc_params.recv().await.unwrap();
    server.stop(true).await;
    println!("server result: {:?}", res);
}
