use crate::routes::{health_check, subscribe};
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::{PgPool};
use env_logger::Env;


pub fn run(listner: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {

    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    let connection = web::Data::new(connection);
    let server = HttpServer::new( move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(connection.clone())
    }).listen(listner)?
    .run();
    Ok(server)
}
