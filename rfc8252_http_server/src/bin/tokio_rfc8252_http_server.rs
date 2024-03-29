use rfc8252_http_server::tokio_start;

#[tokio::main]
async fn main() {
    let res = tokio_start().await;
    println!("server result: {:?}", res);
}
