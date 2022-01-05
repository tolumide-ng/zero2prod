use actix_web::{HttpResponse, web};
use actix_web::http::header::{HeaderMap};
use anyhow::Context;
use sqlx::PgPool;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email::email_client::EmailClient;
use crate::routes::newsletter::helper::{ConfirmedSubscriber, BodyData};
use crate::errors::publish_error::PublishError;
use crate::helpers::authentication::{Credentials};


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
    request: web::HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _credentials = basic_authentication(request.headers())
        .map_err(PublishError::AuthError)?;

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


fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    todo!()
}