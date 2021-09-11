use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, Executor, PgConnection, PgPool};
// use std::detect::features;
// use std::lazy::Lazy;
use std::{net::TcpListener};
use uuid::Uuid;
use once_cell::sync::Lazy;


static TRACKING: Lazy<()> = Lazy::new(|| {
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
    db_pool: PgPool,
    address: String,
}

/// Spin up an instance of our application
/// and returns an address e.g. (http://127.0.0.1:XXXX)
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACKING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = startup::run(listener, connection_pool.clone())
        .expect("Failed to connect to Postgres");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}


pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create Database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection.execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name)).await.expect("Failed to create database");
    
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string()).await.expect("Failed to connect Postgres.");
    sqlx::migrate!("./migrations").run(&connection_pool).await.expect("Failed to migrate the database");

    connection_pool
}


#[actix_rt::test]
async fn health_check_works() {
    // arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/health_check", &app.address)).send().await.expect("Failed to execuet request");

    // Act
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}



#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20example&email=name%40example.com";

    // Act
    let response = client.post(&format!("{}/subscribe", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());


    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "name@example.com");
    assert_eq!(saved.name, "le example");
}



#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=sample", "missing the email"),
        ("email=sample%40email.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act 
        let response = client.post(&format!("{}/subscribe", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(400, response.status().as_u16(), 
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.", error_message
    )
    }
}