use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use secrecy::Secret;
use sqlx::PgPool;

use crate::errors::auth_error::{AuthError, LoginError};
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
pub async fn login(form: web::Form<FormData>, pool: web::Data<PgPool>) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool).await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
    .insert_header((LOCATION, "/"))
    .finish())
}
