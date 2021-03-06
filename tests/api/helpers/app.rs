use zero2prod::configuration::{settings::get_configuration};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::application::{Application, get_connection_pool};
use uuid::Uuid;
use once_cell::sync::Lazy;
use sqlx::{PgPool};
use wiremock::MockServer;

use crate::helpers::db::configure_database;
use crate::helpers::email::ConfirmationLinks;
use crate::helpers::user::TestUser;

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
    pub test_user: TestUser,
    pub api_client: reqwest::Client
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send().await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

          // Extract the link from on eof the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link =         links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // let raw_confirmation_link = raw_link(&body["HtmlBody"].as_str().unwrap());
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text}
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        // destructure username and password of the test_user
        let TestUser {username, password, ..} = &self.test_user;

        self.api_client
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
        where Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await.expect("Failed to execute request.")
    }

    pub async fn get_login(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/password", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response 
        where 
            Body: serde::Serialize
    {
        self.api_client
            .post(&format!("{}/admin/password", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/lopgout", &self.address))
            .send()
            .await
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

    let db_pool = get_connection_pool(&configuration.database);
    let mut test_user = TestUser::generate();
    test_user.store(&db_pool);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address,
        db_pool,
        email_server,
        port: application_port,
        test_user,
        api_client: client,
    };

    test_app
}

