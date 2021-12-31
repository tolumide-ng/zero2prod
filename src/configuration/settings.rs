use std::convert::{TryInto};
use crate::configuration::{
    database_settings::DatabaseSettings,
    application_settings::ApplicationSettings,
    email_settings::EmailClientSettings,
    environment::Environment,
};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_dir = base_path.join("configuration");

    settings.merge(config::File::from(configuration_dir.join("base")).required(true))?;

    // let environment: Environment = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into()).try_into().expect("Failed to parse APP ENVIRONMENT");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    settings.merge(config::File::from(configuration_dir.join(environment.as_str())).required(true))?;

    settings.merge(config::Environment::with_prefix("app").separator("__"))?;

    settings.try_into()
}
