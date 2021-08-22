// use zero2prod::run;
use zero2prod::startup::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("1276.0.0.1:0").expect("Failed to bind random port");

    run(listener)?.await
}

