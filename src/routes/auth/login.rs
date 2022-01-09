use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;


use crate::helpers::auth::{Credentials, validate_credentials};


#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>
}


pub async fn login_form() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../pages/login.html"))
}


#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &traicng::field::display(&user_id));
            HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish()
        }
        Err(_) => {
            todo!()
        }
    }
}
