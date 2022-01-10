use anyhow::Context;
use pbkdf2::{password_hash::{
    rand_core::OsRng,
    PasswordHash, PasswordHasher, SaltString
}, Pbkdf2};
use uuid::Uuid;
use sqlx::PgPool;
use zero2prod::errors::publish_error::PublishError;

pub struct TestUser {
    user_id: Option<Uuid>,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: None,
            username: Uuid::new_v4().to_string(),
            password: "averyhardpassword".into()
        }
    }

    pub async fn store(&mut self, pool: &PgPool) {
        // let hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Pbkdf2
            .hash_password(self.password.as_bytes(), &salt)
            .context("Failed to hash password").map_err(PublishError::UnexpectedError).unwrap().to_string();

        // let hash = format!("{:x}", hash);

        dbg!(&password_hash);
        
        let parsed_hash = PasswordHash::new(&password_hash)
            .context("Error parsing hash")
            .map_err(PublishError::UnexpectedError).unwrap().to_string();

        let user = sqlx::query! (
            "INSERT INTO users (username, hash)
            VALUES ($1, $2) RETURNING user_id",
            self.username,
            parsed_hash
        )
            .fetch_one(pool)
            .await
            .expect("Failed to store test user.");

        self.user_id = Some(user.user_id);

    }
}