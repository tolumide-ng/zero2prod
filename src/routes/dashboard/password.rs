use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web::web;
use actix_web_flash_messages::FlashMessage;
use actix_web_flash_messages::IncomingFlashMessages;
use secrecy::ExposeSecret;
use secrecy::Secret;
use std::fmt::Write;

use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};


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
) -> Result<HttpResponse, actix_web::Error> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error("
        You entered two different new passwords - the field value must match.")
        .send();
    }
    todo!()
}
