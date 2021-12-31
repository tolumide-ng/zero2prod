use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use crate::email::email_client::EmailClient;
use crate::{startup::{run as startup}};
use std::net::TcpListener;
use crate::configuration::settings::Settings;

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client.sender().expect("Invalid sender email address.");

    let timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        configuration.email_client.base_url, sender_email, 
        configuration.email_client.authorization_token,
        timeout);

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address).expect("Failed to bind to random port");
    startup::run(listener, connection_pool, email_client)
}