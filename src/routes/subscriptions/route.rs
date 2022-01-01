use crate::{routes::{prelude::*}, domain::subscriber_email::SubscriberEmail, startup::run::ApplicationBaseUrl};
use std::convert::{TryInto, TryFrom};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use tracing;

use crate::domain::{
    subscriber_name::SubscriberName,
    new_subscriber::NewSubscriber,
};
use crate::email::email_client::EmailClient;
use crate::routes::subscriptions::helpers;


impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self {email, name})
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}


#[tracing::instrument (
    name = "Adding a new subscriber"
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>, 
    pool: web::Data<PgPool>, 
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {

    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => {
            return HttpResponse::BadRequest().finish()
        }
    };

    
    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if helpers::send_confirmation_email(
        &email_client, 
        new_subscriber, 
        base_url.0.as_str())
    .await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }

    HttpResponse::Ok().finish()
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: & PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(), 
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e); 
        e
    })?;

    Ok(())
}