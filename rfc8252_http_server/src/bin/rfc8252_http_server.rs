use rfc8252_http_server::start;

fn main() {
    let res = start();
    println!("server result: {:?}", res);
}
