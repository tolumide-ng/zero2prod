use actix_web::{HttpResponse, web};
use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}


#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content
}

pub async fn publish_newsletter(_body: web::Json<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
