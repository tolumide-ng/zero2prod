use crate::{routes::{prelude::*}, domain::subscriber_email::SubscriberEmail, startup::run::ApplicationBaseUrl};
use std::convert::{TryInto, TryFrom};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use tracing;

use crate::domain::{
    subscriber_name::SubscriberName,
    new_subscriber::NewSubscriber,
};
use crate::email::email_client::EmailClient;
use crate::routes::subscriptions::helpers;
use crate::errors::store_token_error::StoreTokenError;
use crate::errors::subscribe_error::SubscribeError;


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
) -> Result<HttpResponse, SubscribeError> {

    let mut transaction = pool.begin()
        .context("Failed to acquire a Postgres connection from the pool")?;

    let new_subscriber = form.0.try_into().map_err( |e| SubscribeError::ValidationError(e))?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .context("Failed to insert new subscriber in the database.")?;

    let subscription_token = helpers::generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .context("Failed to store the confirmation token for a new subscriber.")?;

    transaction.commit()
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    helpers::send_confirmation_email(
        &email_client, 
        new_subscriber, 
        base_url.0.as_str(), 
        &subscription_token)
    .context("Failed to send a confirmation email")?;
    
    Ok(HttpResponse::Ok().finish())
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn 
insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber
) -> Result<Uuid, sqlx::Error> {
    let user = sqlx::query!(r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation') RETURNING id
        "#,
        Uuid::new_v4(), 
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
        .fetch_one(transaction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e); 
            e
        })?;

    Ok(user.id)
}


#[tracing::instrument(
    name = "Store subscription token in the database"
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>, 
    subscriber_id: Uuid, 
    subscription_token: &str
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id)
    VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    )
        .execute(transaction)
        .await
        .map_err(StoreTokenError)?;
        
        Ok(())
}

enum SubscriberStatus {
    Pending,
    Confirmed,
    NotExist
}

impl SubscriberStatus {
    pub fn new(state: &str) -> Self {
        match state {
            "pending_confirmation" => SubscriberStatus::Pending,
            "confirmed" => SubscriberStatus::Confirmed,
            _ => SubscriberStatus::NotExist
        }
    }
}

pub struct SubscriberState {
    status: SubscriberStatus,
    id: Option<Uuid>,
}

#[tracing::instrument(
    name = "Check if subscriber email already exists in the database"
    skip(transaction, new_subscriber)
)]
pub async fn check_subscriber(
    transaction: &mut Transaction<'_, Postgres>, 
    new_subscriber: &NewSubscriber) -> Result<SubscriberState, sqlx::Error> {
    let user = sqlx::query!(
        r#"SELECT email, id, status FROM subscriptions WHERE email = $1"#, 
        new_subscriber.email.as_ref())
        .fetch_one(transaction).await.map_err(|e| {
            tracing::error!("Failed to execute query {}", e); 
            e
        })?;

    Ok(SubscriberState {
        status: SubscriberStatus::new(user.status.as_str()),
        id: Some(user.id)
    })
}