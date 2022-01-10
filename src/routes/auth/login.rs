use actix_web::cookie::Cookie;
use actix_web::error::InternalError;
use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages, Level};
use actix_session::Session;
use secrecy::{Secret};
use std::fmt::Write;
use sqlx::PgPool;

use crate::errors::auth_error::{AuthError, LoginError};
use crate::helpers::auth::{Credentials, validate_credentials};


#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>
}


pub async fn login_form(
    flash_messages: IncomingFlashMessages
) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(
            Cookie::build("_flash", "")
                .max_age(time::Duration::seconds(0))
                .finish()
        )
        .body(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8" http-equiv="content-type" content="text/html">
                <meta http-equiv="X-UA-Compatible" content="IE=edge">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Login</title>
            </head>
            <body>
                {}
                <form action="/login" method="post">
                    <label>
                        Username
                        <input type="text" placeholder="Enter Username" aria-placeholder="Username" name="username">
                    </label>

                    <label>
                        Password
                        <input type="password" name="password" placeholder="Enter Password" aria-placeholder="password">
                    </label>

                    <button type="submit">Login</button>
                </form>
            </body>
            </html>
            "#, error_html
        ))
}


#[tracing::instrument(
    skip(form, pool, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
    session: Session
) -> Result<HttpResponse, InternalError<LoginError>> {

    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            session.renew();
            session.insert("user_id", user_id).map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
        }
        Err(e) => {
            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let e =  match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            FlashMessage::error(e.to_string()).send();

            let response = HttpResponse::SeeOther()
                // .cookie(Cookie::new("_flash", e.to_string()))
                .insert_header((LOCATION, "/login"))
                .finish();

            return Err(login_redirect(e))
        }
    }

    Ok(HttpResponse::SeeOther()
    .insert_header((LOCATION, "/admin/dashboard"))
    .finish())
}


fn login_credentials(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();

    InternalError::from_response(e, response)
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther().insert_header((LOCATION, "/login")).finish();

    InternalError::from_response(e, response)
}