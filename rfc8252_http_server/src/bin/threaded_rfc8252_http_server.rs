use rfc8252_http_server::threaded_start;

fn main() {
    let res = threaded_start();
    println!("server result: {:?}", res);
}
