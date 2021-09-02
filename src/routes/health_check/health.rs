use crate::routes::prelude::*;

pub async fn health_check() -> HttpResponse {
    println!("received a request here");
    HttpResponse::Ok().finish()
}