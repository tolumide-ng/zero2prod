use actix_web::{
    cookie::Key, App, dev::Server, 
    HttpServer, web
};
use actix_web_flash_messages::{
    FlashMessagesFramework,
    storage::CookieMessageStore,
};
use actix_session::SessionMiddleware;
use secrecy::{ExposeSecret, Secret};
use std::net::TcpListener;
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;

use crate::email::email_client::EmailClient;
use crate::routes::{health_check, subscribe, confirm, publish_newsletter, home, login_form, login};

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listner: TcpListener, 
    db_pool: PgPool, 
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>
) -> Result<Server, std::io::Error> {

    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = hmac_secret.expose_secret().as_bytes();

    let message_store = CookieMessageStore::builder(
        Key::from(secret_key)
    ).build();
    
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let server = HttpServer::new( move || {
        App::new()
        .wrap(message_framework.clone())
        .wrap(SessionMiddleware::new(todo!(), secret_key.clone()))
        .wrap(TracingLogger::default())
            .wrap(message_framework.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    }).listen(listner)?
    .run();
    Ok(server)
}
