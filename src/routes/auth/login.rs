use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use secrecy::Secret;


#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    // password: Secret<String>
}

pub async fn login_form() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../pages/login.html"))
}



pub async fn login(_from: web::Form<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
