use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email::email_client::EmailClient;
use crate::errors::auth_error::AuthError;
use crate::routes::newsletter::helper::{ConfirmedSubscriber, BodyData};
use crate::errors::publish_error::PublishError;
use crate::helpers::auth::{basic_authentication, validate_credentials};


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


#[tracing::instrument(
    name = "Publish a neslietter issue",
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: web::HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers())
        .map_err(PublishError::AuthError)?;

        
        tracing::Span::current().record(
            "username", 
            &tracing::field::display(&credentials.username));
            
        let user_id = validate_credentials(credentials, &pool).await
            .map_err(|e| match e {
                AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into())
            })?;

        tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
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
