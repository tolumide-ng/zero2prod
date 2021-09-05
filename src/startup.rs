use crate::routes::{health_check, subscribe};
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;


pub fn run(listner: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {

    let connection = web::Data::new(connection);
    let server = HttpServer::new( move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(connection.clone())
    }).listen(listner)?
    .run();
    Ok(server)
}
