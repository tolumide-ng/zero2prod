use crate::routes::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;
use tracing;
use chrono::Utc;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    tracing::info!("Correlation Id: {} - Adding {} {} as a new subscriber", request_id, form.name, form.email);
    tracing::info!("Correlation Id: {} - Saving new subscriber's details in the database", request_id);

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) 
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    ).execute(pool.get_ref()).await {
        Ok(_) => {
            tracing::info!("Correlation Id: {} - New subscriber detail saved", request_id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Correlation Id: {} - Failed to execute query {:#?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

