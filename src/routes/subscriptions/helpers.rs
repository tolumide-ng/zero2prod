use tracing;
use crate::email::email_client::EmailClient;
use crate::domain::new_subscriber::NewSubscriber;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient, 
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, subscription_token);
    let html_body =     &format!("Welcome to our newsletter!<br />`
        Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link);
    let plain_body = &format!("Welcome to our newsletter!\nVisit {} to confirm your subscription.", confirmation_link);

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body).await
}


/// Generate a random 25-characters-long case-sensitive subscription token.
pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
