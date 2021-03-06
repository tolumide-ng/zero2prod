use actix_web::{
    cookie::Key, App, dev::Server, 
    HttpServer, web,
};
use actix_web_flash_messages::{
    FlashMessagesFramework,
    storage::CookieMessageStore,
};
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use secrecy::{ExposeSecret, Secret};
use std::net::TcpListener;
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;

use crate::email::email_client::EmailClient;
use crate::routes::{health_check, subscribe, confirm, publish_newsletter, 
    home, login_form, login, admin_dashboard, change_password, change_password_form, log_out};

pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listner: TcpListener, 
    db_pool: PgPool, 
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {

    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let server = HttpServer::new( move || {
        App::new()
        .wrap(message_framework.clone())
        .wrap(SessionMiddleware::new(redis_store.clone(), secret_key.clone()))
        .wrap(TracingLogger::default())
            .wrap(message_framework.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/admin/dashboard", web::get().to(admin_dashboard))
            .route("/admin/password", web::get().to(change_password_form))
            .route("/admin/password", web::post().to(change_password))
            .route("/admin/logout", web::post().to(log_out))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    }).listen(listner)?
    .run();
    Ok(server)
}
