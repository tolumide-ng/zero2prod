use actix_web::http::header::{HeaderMap};
use secrecy::{Secret};
use anyhow::Context;
use pbkdf2::password_hash::{PasswordVerifier, PasswordHash};
use pbkdf2::Pbkdf2;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::telemetry::spawn_blocking_with_tracing;
use crate::errors::auth_error::AuthError;


const DUMMY_PASSWORD: &str = "$pbkdf2-sha512$i=10000$O484sW7giRw+nt5WVnp15w$jEUMVZ9adB+63ko/8Dr9oB1jWdndpVVQ65xRlT+tA1GTKcJ7BWlTjdaiILzZAhIPEtgTImKvbgnu8TS/ZrjKgA";



pub struct Credentials {
    pub username: String,
    pub password: Secret<String>
}


pub fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authentication")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic")
        .context("The authorization scheme was not 'Basic'.")?;

    let decoded_bytes = base64::decode_config(base64encoded_segment, base64::STANDARD)
        .context("Failed to base64-decode 'Basic' credentials")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    
    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {username, password: Secret::new(password)})
}



#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool
) -> Result<uuid::Uuid, AuthError> {

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
    .map_err(AuthError::UnexpectedError)??;

    user_id.ok_or_else(|| anyhow::anyhow!("Unknown username"))
        .map_err(AuthError::InvalidCredentials)
    
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
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(AuthError::UnexpectedError)?;

    Pbkdf2.verify_password(password_candidate.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}
