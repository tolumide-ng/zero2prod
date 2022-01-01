use actix_web::dev::Server;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::email::email_client::EmailClient;
use crate::{startup::run::run};
use std::net::TcpListener;
use crate::configuration::{
    settings::Settings,
    database_settings::DatabaseSettings,
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
    
        let connection_pool = get_connection_pool(&configuration.database);   
    
        let sender_email = configuration
            .email_client.sender().expect("Invalid sender email address.");
    
        let timeout = configuration.email_client.timeout();
    
        let email_client = EmailClient::new(
            configuration.email_client.base_url, sender_email, 
            configuration.email_client.authorization_token,
            timeout);
    
        let address = format!("{}:{}", 
            configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        
        let server = run(
            listener, 
            connection_pool, 
            email_client, 
            configuration.application.base_url)?;
        Ok(Self {port, server})
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(configuration.with_db())
}
