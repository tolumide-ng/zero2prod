// use zero2prod::run;
use zero2prod::startup::run;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configurations");
    let connection_pool = PgPool::connect(&configuration.database.connection_string()).await.expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    run(listener, connection_pool)?.await
}

