use crate::routes::prelude::*;

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}