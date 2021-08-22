use crate::routes::health_check::health::health_check;
use crate::routes::subscriptions::subscribe::subscribe;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use std::net::TcpListener;


pub fn run(listner: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new( || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
    }).listen(listner)?
    .run();
    Ok(server)
}
