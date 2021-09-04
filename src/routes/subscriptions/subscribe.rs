use crate::routes::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;
use tracing;
use chrono::Utc;
use tracing_futures::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    // tracing::info!("Correlation Id: {} - Adding {} {} as a new subscriber", request_id, form.name, form.email);
    // tracing::info!("Correlation Id: {} - Saving new subscriber's details in the database", request_id);

    let request_span = tracing::info_span!("Adding a new subscriber", %request_id, subscriber_email=%form.email, subscriber_name=%form.name);

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) 
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    ).execute(pool.get_ref()).instrument(query_span).await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Failed to execute query {:#?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

