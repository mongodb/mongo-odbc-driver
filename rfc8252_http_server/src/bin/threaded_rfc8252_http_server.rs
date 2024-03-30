use actix_web::rt;
use rfc8252_http_server::threaded_start;

fn main() {
    let (server, oidc_params_channel) = threaded_start();
    let res = oidc_params_channel.recv().unwrap();
    rt::System::new().block_on(server.stop(true));
    println!("server result: {:?}", res);
}
