use actix_web::{web, HttpResponse};
use actix_session::Session;

fn e500<T>(e: T) -> actix_web::error::InternalError<T> {
    actix_web::error::InternalError::from_response(e, HttpResponse::InternalServerError().finish())
}


pub async fn admin_dashboard(
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get::<Uuid>("user_id")
        .map_err(e500)? 
    {
        todo!()
    } else {
        todo!()
    }

    Ok(HttpResponse::Ok().finish())
}