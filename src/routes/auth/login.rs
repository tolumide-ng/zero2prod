use actix_web::error::InternalError;
use actix_web::{HttpResponse, web};
use actix_web::http::header::{ContentType, LOCATION};
use hmac::{Hmac, Mac, NewMac};
use secrecy::{Secret, ExposeSecret};
use sha2::Sha256;
use sqlx::PgPool;

use crate::configuration::application_settings::HmacSecret;
use crate::errors::auth_error::{AuthError, LoginError};
use crate::helpers::auth::{Credentials, validate_credentials};


#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
    tag: Option<String>,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag.unwrap())?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error.unwrap()));
        let mut mac = HmacSha256::new_from_slice(secret.0.expose_secret().as_bytes());
        mac.verify(&tag)?;

        Ok(self.error.unwrap())
    }
}

type HmacSha256 = Hmac<Sha256>;


pub async fn login_form(
    query: web::Query<Option<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error_html = match query.0 {
        None => "".into(),
        Some(query) => match query.verify(&secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(error.message = %e, error.cause_chain = ?e, 
                "Failed to veridy query parameters using the HMAC tag")
                "".into()
            }
        } 
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
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
) -> Result<HttpResponse, LoginError> {

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
                .insert_header((LOCATION, "/login")).finish();

            return Err(InternalError::from_response(e, response))
        })?;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
    .insert_header((LOCATION, "/"))
    .finish())
}
