mod health_check;
mod subscriptions;
mod prelude;
mod subscriptions_confirm;
mod newsletter;
mod auth;
mod pages;
mod dashboard;

pub use health_check::route::health_check;
pub use subscriptions::route::subscribe;
pub use subscriptions_confirm::route::confirm;
pub use newsletter::route::publish_newsletter;
pub use pages::home::home;
pub use auth::login::login_form;
pub use auth::login::login;
pub use dashboard::admin_dashboard::admin_dashboard;