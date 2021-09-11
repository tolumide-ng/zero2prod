use zero2prod::telemetry::init_subscriber;
use zero2prod::{startup::run, telemetry::get_subscriber};
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;
use std::net::TcpListener;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    
    let configuration = get_configuration().expect("Failed to read configurations");
    let connection_pool = PgPool::connect(&configuration.database.connection_string()).await.expect("Failed to connect to Postgres");
    let address = format!("{}:{}", configuration.application_port.host, configuration.application_port.port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    run(listener, connection_pool)?.await?;
    Ok(())
}

