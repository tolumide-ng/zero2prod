use secrecy::ExposeSecret;
use sqlx::{PgPool};
use anyhow::Context;
use crate::helpers::authentication::Credentials;
use crate::errors::publish_error::PublishError;

pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, PublishError> {
    let user_id: Option<_> = sqlx::query!(
        r#"
        SELECT user_id
        FROM users
        WHERE username = $1 AND password = $2
        "#,
        credentials.username,
        credentials.password.expose_secret()
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to validate auth credentials.")
    .map_err(PublishError::UnexpectedError)?;

    user_id
        .map(|row| row.user_id)
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password."))
        .map_err(PublishError::AuthError)
}