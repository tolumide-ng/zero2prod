use uuid::Uuid;
use zero2prod::configuration::{database_settings::DatabaseSettings};
use sqlx::{Connection, Executor, PgConnection, PgPool};


pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create Database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection.execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await.expect("Failed to create database");
    
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await.expect("Failed to connect Postgres.");
    
    sqlx::migrate!("./migrations").run(&connection_pool)
        .await.expect("Failed to migrate the database");

    connection_pool
}
