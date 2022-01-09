use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::Secret;
use std::net::TcpListener;
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;

use crate::configuration::application_settings::HmacSecret;
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

    let mesage_framework = FlashMessageFranework::builder(todo!()).build();

    let server = HttpServer::new( move || {
        App::new()
        .wrap(TracingLogger::default())
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
            .app_data(web::Data::new(hmac_secret.clone()))
    }).listen(listner)?
    .run();
    Ok(server)
}
