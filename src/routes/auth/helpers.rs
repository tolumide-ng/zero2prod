use anyhow::Context;
use pbkdf2::password_hash::{PasswordVerifier, PasswordHash};
use pbkdf2::Pbkdf2;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use secrecy::Secret;

use crate::helpers::authentication::Credentials;
use crate::errors::publish_error::PublishError;
use crate::telemetry::spawn_blocking_with_tracing;


const DUMMY_PASSWORD: &str = "$pbkdf2-sha512$i=10000$O484sW7giRw+nt5WVnp15w$jEUMVZ9adB+63ko/8Dr9oB1jWdndpVVQ65xRlT+tA1GTKcJ7BWlTjdaiILzZAhIPEtgTImKvbgnu8TS/ZrjKgA";


#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, PublishError> {

    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
    DUMMY_PASSWORD.to_string()
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)??;

    user_id.ok_or_else(|| PublishError::AuthError(anyhow::anyhow!("Unknown username.")))
    
}


#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, hash
        FROM users
        WHERE username = $1
        "#,
        username
    )
        .fetch_optional(pool)
        .await
        .context("Failed to perform a query to retrieve stored credentials.")?
        .map(|row| (row.user_id, Secret::new(row.hash)));

        Ok(row)
}


#[tracing::instrument(name = "Verify password hash", skip(expected_password_hash, password_candidate))]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Pbkdf2.verify_password(password_candidate.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
        .map_err(PublishError::AuthError)
}
