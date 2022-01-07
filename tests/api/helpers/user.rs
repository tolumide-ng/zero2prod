use sha3::Digest;
use uuid::Uuid;
use sqlx::PgPool;

pub struct TestUser {
    pub user_id: Option<Uuid>,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: None,
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&mut self, pool: &PgPool) {
        let hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let hash = format!("{:x}", hash);
        let user = sqlx::query!(
            "INSERT INTO users (username, hash)
            VALUES ($1, $2) RETURNING user_id",
            self.username,
            hash
        )
            .fetch_one(pool)
            .await
            .expect("Failed to store test user.");

        self.user_id = Some(user.user_id);

    }
}