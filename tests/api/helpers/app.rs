use zero2prod::configuration::{settings::get_configuration};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::application::{Application, get_connection_pool};
use uuid::Uuid;
use once_cell::sync::Lazy;
use sqlx::{PgPool};
use wiremock::MockServer;

use crate::helpers::db::configure_database;

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
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send().await
            .expect("Failed to execute request.")
    }
}


/// Spin up an instance of our application
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;
    
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        // let port = listener.local_addr().unwrap().port();
        // Use a random OS port 
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await.expect("Failed to build application");
    let application_port = application.port();

    let address = format!("http://127.0.0.1:{}", application_port);
    
    // build(configuration).await.expect("Failed to build expectation");
    let _ = tokio::spawn(application.run_until_stopped());



    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        port: application_port,
    }
}

