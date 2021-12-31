use crate::routes::{health_check, subscribe, confirm};
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;
use crate::email::email_client::EmailClient;



pub fn run(listner: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {

    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new( move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    }).listen(listner)?
    .run();
    Ok(server)
}
