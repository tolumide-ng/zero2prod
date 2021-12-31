use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{postgres::{PgConnectOptions, PgSslMode}};


#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };

        PgConnectOptions::new().host(&self.host).username(&self.username).password(&self.password).port(self.port).ssl_mode(ssl_mode)
    }

    
    pub fn with_db(&self) -> PgConnectOptions {
        // self.without_db().database(&self.database_name).log_statements(log::LevelFilter::Trace).clone()
        self.without_db().database(&self.database_name)
    }
}
