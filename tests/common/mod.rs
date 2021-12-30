use zero2prod::configuration::{
    settings::get_configuration, 
    database_settings::DatabaseSettings,
};
use zero2prod::email::email_client::EmailClient;
use zero2prod::startup;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use std::{net::TcpListener};
use uuid::Uuid;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};

static TRACING: Lazy<()> = Lazy::new(|| {
    // Lazy::force(this)
    let subscriber_name = "test".to_string();
    let default_filter_level = "debug".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub db_pool: PgPool,
    pub address: String,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create Database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection.execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name)).await.expect("Failed to create database");
    
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db()).await.expect("Failed to connect Postgres.");
    
    sqlx::migrate!("./migrations").run(&connection_pool).await.expect("Failed to migrate the database");

    connection_pool
}

/// Spin up an instance of our application
/// and returns an address e.g. (http://127.0.0.1:XXXX)
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let sender_email = configuration.email_client.sender().expect("Invalid sender email address.");

    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);

    let server = startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to connect to Postgres");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

