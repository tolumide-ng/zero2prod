use anyhow::Context;
use pbkdf2::password_hash::{PasswordVerifier, PasswordHash};
use pbkdf2::Pbkdf2;
use secrecy::ExposeSecret;
use sqlx::{PgPool};

use crate::helpers::authentication::Credentials;
use crate::errors::publish_error::PublishError;

pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, PublishError> {

    // let salt = SaltString::generate(&mut OsRng);
    // let phc_hash = Pbkdf2.hash_password(&[39], &salt).unwrap().to_string();
    // let parsed_hash = PasswordHash::new(&phc_hash)
    //     .context("Failed to parse the error").map_err(PublishError::UnexpectedError)?;

    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, hash
        FROM users
        WHERE username = $1
        "#,
        credentials.username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")
    .map_err(PublishError::UnexpectedError)?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.hash, row.user_id),
        None => {
            return Err(PublishError::AuthError(anyhow::anyhow!("Unknown username")))
        }
    };

    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;


    Pbkdf2.verify_password(credentials.password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid Password")
        .map_err(PublishError::AuthError)?;

    Ok(user_id)
}


