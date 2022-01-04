use actix_web::{HttpResponse, ResponseError, web};
use anyhow::Context;
use sqlx::PgPool;
use actix_web::http::StatusCode;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email::email_client::EmailClient;

use crate::errors::helper::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}


#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}


impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Get confirmed subscribers", skip(pool)
)]
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {

    struct Row {
        email: String
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT email 
        FROM subscriptions 
        WHERE status = 'confirmed'
        "#,
    ).fetch_all(pool)
        .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .filter_map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Some(ConfirmedSubscriber {email}),
            Err(error) => {
                tracing::warn!("A confirmed subscriber is using an invalid email address.\n{}", error);
                None
            }
        }).collect();

    Ok(confirmed_subscribers)
}

pub async fn publish_newsletter(
    body: web::Json<BodyData>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        email_client.send_email(
            subscriber.email,
            &body.title,
            &body.content.html,
            &body.content.text
        ).await
            .with_context(|| {
                format!("Failed to sned newsletter issue to {}", subscriber.email)
            })?;
    }
    Ok(HttpResponse::Ok().finish())
}
