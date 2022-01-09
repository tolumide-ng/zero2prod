use actix_web::cookie::Cookie;
use actix_web::error::InternalError;
use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use secrecy::{Secret};
use sqlx::PgPool;

use crate::errors::auth_error::{AuthError, LoginError};
use crate::helpers::auth::{Credentials, validate_credentials};


#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>
}


pub async fn login_form(
    request: web::HttpRequest
) -> HttpResponse {
    let error_html = match request.cookie("_flash")  {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", htmlescape::encode_minimal(cookie.value()))
        } 
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(
            Cookie::build("_flash", "")
                .max_age(time::Duration::zero())
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
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<LoginError>> {

    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool).await
        .map_err(|e| {
            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                let mut mac = HmacSha256::new_from_slice(secret.expose_secret()).as_bytes().unwrap();;
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let e =  match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                // .insert_header(("Set-Cookie", format!("_flash={}", e)))
                .cookie(Cookie::new("_flash", e.to_string()))
                .finish();

            return Err(InternalError::from_response(e, response))
        });

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id.unwrap()));

    Ok(HttpResponse::SeeOther()
    .insert_header((LOCATION, "/"))
    .finish())
}
