use actix_session::Session;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web::web;
use actix_web_flash_messages::FlashMessage;
use actix_web_flash_messages::IncomingFlashMessages;
use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::PgPool;
use std::fmt::Write;

use crate::helpers::auth::Credentials;
use crate::helpers::auth::validate_credentials;
use crate::routes::dashboard::admin_dashboard::get_username;
use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};
use crate::errors::auth_error::AuthError;


pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Change Password</title>
        </head>
        <body>
            {}
            <form action="/admin/password" method="post">
            <label>Current password
            <input
            type="password"
            placeholder="Enter current password"
            name="current_password"
            />
            </label>
            <br />
            <label>New password
            <input
            type="password"
            placeholder="Enter new password"
            name="new_password"
            />
            </label>
            <br />
            <label>Confirm new password
            <input
            type="password"
            placeholder="Type the new password again"
            name="new_password_check"
            />
            </label>
            <br />
            <button type="submit">Change password</button>
            </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>"#, msg_html
    )))
}

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error("
        You entered two different new passwords - the field value must match.")
        .send();
    }

    let user_id = session.get_user_id().map_err(e500)?;
    if user_id.is_none() {
        return Ok(see_other("/login"));
    };
    let user_id = user_id.unwrap();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        // 
    } 

    let username = get_username(user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        }
    }

    todo!()
}



