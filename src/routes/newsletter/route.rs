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
async fn get_confirmed_subscribers(pool: &PgPool) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {

    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email 
        FROM subscriptions 
        WHERE status = 'confirmed'
        "#,
    ).fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber {email}),
            Err(error) => {
                tracing::warn!("A confirmed subscriber is using an invalid email address.\n{}", error);
                Err(anyhow::anyhow!(error))
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
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                    &subscriber.email,
                    &body.title,
                    &body.content.html,
                    &body.content.text
                )
                .await
                .with_context(|| {
                        format!(
                            "Failed to sned newsletter issue to {}", subscriber.email
                        )
                    })?;

            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                )

            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}
