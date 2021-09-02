use crate::routes::prelude::*;
use sqlx::PgPool;
use log;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    log::info!("Correlation Id: {} - Adding {} {} as a new subscriber", request_id, form.name, form.email);
    log::info!("Correlation Id: {} - Saving new subscriber's details in the database", request_id);

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (email, name) 
        VALUES ($1, $2)
        "#,
        form.email,
        form.name
    ).execute(pool.get_ref()).await {
        Ok(_) => {
            log::info!("Correlation Id: {} - New subscriber detail saved", request_id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            log::error!("Correlation Id: {} - Failed to execute query {:#?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

