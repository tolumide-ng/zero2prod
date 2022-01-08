use argon2::{Algorithm, Argon2, Version, Params};
use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::{PgPool};
use argon2::PasswordHasher;

use crate::helpers::authentication::Credentials;
use crate::errors::publish_error::PublishError;

pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, PublishError> {
    // let hash = sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2,1, None)
            .context("Failed to build Argon2 parameters")
            .map_err(PublishError::UnexpectedError)?,
    );

    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, hash, salt
        FROM users
        WHERE username = $1
        "#,
        credentials.username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")
    .map_err(PublishError::UnexpectedError)?;

    let (expected_password_hash, user_id, salt) = match row {
        Some(row) => (row.hash, row.id, row.salt),
        None => {
            return Err(PublishError::AuthError(anyhow::anyhow!("Unknown username")))
        }
    };

    let hash = hasher
        .hash_password(credentials.password.expose_secret().as_bytes(), &salt)
        .context("Failed to hash password")
        .map_err(PublishError::UnexpectedError)?;

    let hash = format!(":x", hash.hash.unwrap());

    if hash != expected_password_hash {
        Err(PublishError::AuthError(anyhow::anyhow!("Invalid password.")))
    } else {
        Ok(user_id)
    }
}


