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

    let subscriber_id = match insert_subscriber(&pool, &new_subscriber).await {
        Ok(subscriber_id) => {
            subscriber_id
        }
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    let subscription_token = helpers::generate_subscription_token();
    
    if store_token(&pool, subscriber_id, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }

    if helpers::send_confirmation_email(
        &email_client, 
        new_subscriber, 
        base_url.0.as_str(), 
        &subscription_token)
    .await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }

    HttpResponse::Ok().finish()
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: & PgPool, new_subscriber: &NewSubscriber) -> Result<Uuid, sqlx::Error> {
    let user = sqlx::query!(r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation') RETURNING id
        "#,
        Uuid::new_v4(), 
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e); 
            e
        })?;

    Ok(user.id)
}


#[tracing::instrument(
    name = "Store subscription token in the database"
    skip(subscription_token, pool)
)]
pub async fn store_token(pool: &PgPool, subscriber_id: Uuid, subscription_token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id,
    ).execute(pool).await.map(|e| {
        tracing::error!("Failed to execute query: {:?}", e)
    })
}