use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;
use crate::{session_state::TypedSession, utils::{see_other, e500}};

pub async fn log_out(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        Ok(see_other("/login"))
    } else {
        FlashMessage::info("You have successfully logged out.").send();
    }
    
    Ok(see_other("/login"))
}